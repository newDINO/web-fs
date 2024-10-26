use std::{cell::RefCell, io::Result, path::Path, rc::Rc};

use crate::{open_options::OpenFileFuture, read::ReadResult, OpenOptions, Task, FS};

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
    pub fn create_new<P: AsRef<Path>>(path: P) -> OpenFileFuture {
        OpenOptions::new().create_new(true).write(true).open(path)
    }
}

impl Drop for File {
    fn drop(&mut self) {
        FS.with_borrow(|fs| fs.drop_file(self.fd));
    }
}
