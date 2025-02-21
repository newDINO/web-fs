use std::{
    cell::RefCell,
    future::Future,
    io::{Error, ErrorKind, Result},
    path::Path,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use futures_lite::AsyncWriteExt;
use js_sys::Object;
use wasm_bindgen::JsValue;

use crate::{
    open_options::OpenFileFuture, read::ReadResult, util::set_value, FileType, Fs, Metadata,
    OpenOptions, Permissions, Task, FD, FS, INDEX, SIZE, TRUNCATE,
};

pub struct File {
    pub(crate) fd: usize,
    pub(crate) cursor: u64,
    pub(crate) size: u64,
    pub(crate) read_task: Option<Rc<RefCell<Task<Result<ReadResult>>>>>,
    pub(crate) write_task: Option<Rc<RefCell<Task<Result<usize>>>>>,
    pub(crate) flush_task: Option<Rc<RefCell<Task<Result<()>>>>>,
    pub(crate) close_task: Option<Rc<RefCell<Task<Result<()>>>>>,
}
impl File {
    pub(crate) fn new(fd: usize, size: u64) -> Self {
        Self {
            fd,
            size,
            cursor: 0,
            read_task: None,
            write_task: None,
            flush_task: None,
            close_task: None,
        }
    }
    pub fn open<P: AsRef<Path>>(path: P) -> OpenFileFuture {
        OpenOptions::new().read(true).open(path)
    }
    pub fn create<P: AsRef<Path>>(path: P) -> OpenFileFuture {
        OpenOptions::new().create(true).write(true).open(path)
    }
    /// Currenly this is only available in [`std::fs::File`] not [`async-fs::File`]
    pub fn create_new<P: AsRef<Path>>(path: P) -> OpenFileFuture {
        OpenOptions::new().create_new(true).write(true).open(path)
    }
    /// This is a different implementation that require `&mut self` while [`async-fs::File`] and [`std::fs::File`] doesn't.
    pub async fn sync_data(&mut self) -> Result<()> {
        self.flush().await?;
        Ok(())
    }
    /// This is a different implementation that require `&mut self` while [`async-fs::File`] and [`std::fs::File`] doesn't.
    pub async fn sync_all(&mut self) -> Result<()> {
        self.flush().await?;
        Ok(())
    }
    /// This is a different implementation that require `&mut self` while [`async-fs::File`] and [`std::fs::File`] doesn't.
    pub fn set_len<'a>(&'a mut self, size: u64) -> TruncateFuture<'a> {
        let task = Rc::new(RefCell::new(Task {
            waker: None,
            result: None,
        }));
        let task_clone = task.clone();
        FS.with_borrow(|fs| fs.truncate(self.fd, size, task_clone));
        TruncateFuture {
            task,
            size,
            file: self,
        }
    }
    pub async fn metadata(&self) -> Result<Metadata> {
        Ok(Metadata {
            ty: FileType::File,
            len: self.size,
        })
    }
    /// Currently always returns Err,
    /// because the permission of a file in *File System API* is determined when opening the file
    /// and can't be changed afterwards.
    pub async fn set_permissions(&self, perm: Permissions) -> Result<()> {
        drop(perm);
        Err(Error::from(ErrorKind::Other))
    }
}

impl Drop for File {
    fn drop(&mut self) {
        FS.with_borrow(|fs| fs.drop_file(self.fd));
    }
}

pub struct TruncateFuture<'a> {
    task: Rc<RefCell<Task<Result<()>>>>,
    size: u64,
    file: &'a mut File,
}
impl Future for TruncateFuture<'_> {
    type Output = Result<()>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let inner_self = self.get_mut();
        let mut inner = inner_self.task.borrow_mut();

        if let Some(val) = inner.result.take() {
            if let Ok(()) = val {
                inner_self.file.size = inner_self.size;
            }
            return Poll::Ready(val);
        }
        inner.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}

impl Fs {
    fn truncate(&self, fd: usize, size: u64, task: Rc<RefCell<Task<Result<()>>>>) {
        let index = self.inner.borrow_mut().truncating_tasks.insert(task);

        let msg = Object::new();
        let truncate = Object::new();
        set_value(&truncate, &INDEX, &JsValue::from_f64(index as f64));
        set_value(&truncate, &FD, &JsValue::from_f64(fd as f64));
        set_value(&truncate, &SIZE, &JsValue::from_f64(size as f64));
        set_value(&msg, &TRUNCATE, &truncate);

        self.worker.post_message(&msg).unwrap();
    }
}
