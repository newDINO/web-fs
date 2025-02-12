#![doc = include_str!("../README.md")]

mod open_options;
use arena::Arena;
use js_sys::{ArrayBuffer, JsString, Object, Reflect};
pub use open_options::OpenOptions;
use read::ReadResult;
use util::{get_value, get_value_as_f64, js_value_to_error, set_value, Task};
use wasm_bindgen_futures::{stream::JsStream, JsFuture};
mod arena;
mod file;
mod read;
mod seek;
mod write;
pub use file::File;
mod metadata;
mod util;
pub use metadata::*;

use std::{
    cell::RefCell,
    ffi::OsString,
    io::{Error, ErrorKind, Result},
    path::{Component, Path, PathBuf},
    rc::Rc,
};

use futures_lite::{AsyncReadExt, AsyncWriteExt, Stream, StreamExt};
use wasm_bindgen::prelude::*;
use web_sys::{
    window, FileSystemDirectoryHandle, FileSystemFileHandle, FileSystemGetDirectoryOptions,
    FileSystemGetFileOptions, FileSystemRemoveOptions, MessageEvent, StorageManager, Worker,
    WorkerGlobalScope,
};

include!("./c_static_str.rs");

const GETTING_JS_FIELD_ERROR: &str = "Getting js field error, this is an error of the crate.";
const ARENA_REMOVE_ERROR: &str = "Removing from arena error, this is an error of the crate.";
const DYN_INTO_ERROR: &str = "Converting js type failed, this is an error of the crate.";
const POST_ERROR: &str = "Posting message to worker failed, this is an error of the crate";

struct FsInner {
    opening_tasks: Arena<Rc<RefCell<Task<Result<File>>>>>,
    reading_tasks: Arena<Rc<RefCell<Task<Result<ReadResult>>>>>,
    writing_tasks: Arena<Rc<RefCell<Task<Result<usize>>>>>,
    flushing_tasks: Arena<Rc<RefCell<Task<Result<()>>>>>,
    closing_tasks: Arena<Rc<RefCell<Task<Result<()>>>>>,
    truncating_tasks: Arena<Rc<RefCell<Task<Result<()>>>>>,
}
struct Fs {
    inner: Rc<RefCell<FsInner>>,
    _closure: Closure<dyn FnMut(MessageEvent)>,
    worker: Worker,
}
impl Fs {
    fn new() -> Self {
        let worker = Worker::new(&wasm_bindgen::link_to!(module = "/src/worker.js"))
            .expect("Creating web worker failed. This crate relis on web worker to work.");

        let inner = FsInner {
            opening_tasks: Arena::new(),
            reading_tasks: Arena::new(),
            writing_tasks: Arena::new(),
            flushing_tasks: Arena::new(),
            closing_tasks: Arena::new(),
            truncating_tasks: Arena::new(),
        };
        let inner = Rc::new(RefCell::new(inner));
        let inner_clone = inner.clone();
        #[repr(u32)]
        enum InMsgType {
            Open = 0,
            Read,
            Write,
            Flush,
            Close,
            Truncate,
        }
        let on_message: Closure<dyn FnMut(MessageEvent)> =
            Closure::new(move |msg: MessageEvent| {
                let received = msg.data();
                let error = get_value(&received, &ERROR);
                let error = if !error.is_undefined() {
                    Some(error.as_string()).expect(
                        "Converting js error to string failed, this is an error of the crate.",
                    )
                } else {
                    None
                };

                let open_msg = Reflect::get_u32(&received, InMsgType::Open as u32)
                    .expect(GETTING_JS_FIELD_ERROR);
                if !open_msg.is_undefined() {
                    let index = get_value_as_f64(&open_msg, &INDEX) as usize;
                    let task = inner_clone
                        .borrow_mut()
                        .opening_tasks
                        .remove(index)
                        .expect(ARENA_REMOVE_ERROR);
                    let mut state = task.borrow_mut();
                    if let Some(error) = error {
                        state.result = Some(Err(Error::other(error)));
                    } else {
                        let fd = get_value_as_f64(&open_msg, &FD) as usize;
                        let size = get_value_as_f64(&open_msg, &SIZE) as u64;
                        state.result = Some(Ok(File::new(fd, size)));
                    }
                    if let Some(waker) = state.waker.take() {
                        waker.wake();
                    }
                    return;
                }
                let read_msg = Reflect::get_u32(&received, InMsgType::Read as u32)
                    .expect(GETTING_JS_FIELD_ERROR);
                if !read_msg.is_undefined() {
                    let index = get_value_as_f64(&read_msg, &INDEX) as usize;
                    let task = inner_clone
                        .borrow_mut()
                        .reading_tasks
                        .remove(index)
                        .expect(ARENA_REMOVE_ERROR);
                    let mut state = task.borrow_mut();
                    if let Some(error) = error {
                        state.result = Some(Err(Error::other(error)));
                    } else {
                        let size = get_value_as_f64(&read_msg, &SIZE) as usize;
                        let array_buffer = get_value(&read_msg, &BUF)
                            .dyn_into::<ArrayBuffer>()
                            .expect(DYN_INTO_ERROR);
                        state.result = Some(Ok(ReadResult {
                            buf: array_buffer,
                            size,
                        }));
                    }
                    if let Some(waker) = state.waker.take() {
                        waker.wake();
                    }
                    return;
                }
                let write_msg = Reflect::get_u32(&received, InMsgType::Write as u32)
                    .expect(GETTING_JS_FIELD_ERROR);
                if !write_msg.is_undefined() {
                    let index = get_value_as_f64(&write_msg, &INDEX) as usize;
                    let task = inner_clone
                        .borrow_mut()
                        .writing_tasks
                        .remove(index)
                        .expect(ARENA_REMOVE_ERROR);
                    let mut state = task.borrow_mut();

                    if let Some(error) = error {
                        state.result = Some(Err(Error::other(error)));
                    } else {
                        let size = get_value_as_f64(&write_msg, &SIZE) as usize;
                        state.result = Some(Ok(size));
                    }

                    if let Some(waker) = state.waker.take() {
                        waker.wake();
                    }
                    return;
                }
                let flush_msg = Reflect::get_u32(&received, InMsgType::Flush as u32)
                    .expect(GETTING_JS_FIELD_ERROR);
                if !flush_msg.is_undefined() {
                    let index = get_value_as_f64(&flush_msg, &INDEX) as usize;
                    let task = inner_clone
                        .borrow_mut()
                        .flushing_tasks
                        .remove(index)
                        .expect(ARENA_REMOVE_ERROR);
                    let mut state = task.borrow_mut();

                    if let Some(error) = error {
                        state.result = Some(Err(Error::other(error)));
                    } else {
                        state.result = Some(Ok(()))
                    }

                    if let Some(waker) = state.waker.take() {
                        waker.wake();
                    }
                    return;
                }
                let close_msg = Reflect::get_u32(&received, InMsgType::Close as u32)
                    .expect(GETTING_JS_FIELD_ERROR);
                if !close_msg.is_undefined() {
                    let index = get_value_as_f64(&close_msg, &INDEX) as usize;
                    let task = inner_clone
                        .borrow_mut()
                        .closing_tasks
                        .remove(index)
                        .expect(ARENA_REMOVE_ERROR);
                    let mut state = task.borrow_mut();

                    if let Some(error) = error {
                        state.result = Some(Err(Error::other(error)));
                    } else {
                        state.result = Some(Ok(()))
                    }

                    if let Some(waker) = state.waker.take() {
                        waker.wake();
                    }
                    return;
                }
                let truncate_msg = Reflect::get_u32(&received, InMsgType::Truncate as u32)
                    .expect(GETTING_JS_FIELD_ERROR);
                if !truncate_msg.is_undefined() {
                    let index = get_value_as_f64(&truncate_msg, &INDEX) as usize;
                    let task = inner_clone
                        .borrow_mut()
                        .truncating_tasks
                        .remove(index)
                        .expect(ARENA_REMOVE_ERROR);
                    let mut state = task.borrow_mut();

                    if let Some(error) = error {
                        state.result = Some(Err(Error::other(error)));
                    } else {
                        state.result = Some(Ok(()))
                    }

                    if let Some(waker) = state.waker.take() {
                        waker.wake();
                    }
                    return;
                }
            });
        worker.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        Self {
            inner,
            _closure: on_message,
            worker,
        }
    }
    fn drop_file(&self, fd: usize) {
        let msg = Object::new();
        let drop = Object::new();
        set_value(&drop, &FD, &JsValue::from(fd));
        set_value(&msg, &DROP, &drop);

        self.worker.post_message(&msg).expect(POST_ERROR);
    }
}
thread_local! {
    static FS: RefCell<Fs> = RefCell::new(Fs::new());
}

async fn get_root() -> FileSystemDirectoryHandle {
    let storage = if let Some(window) = window() {
        let navigator = window.navigator();
        navigator.storage()
    } else if js_sys::global().is_instance_of::<WorkerGlobalScope>() {
        let global = js_sys::global().unchecked_into::<WorkerGlobalScope>();
        global.navigator().storage()
    } else {
        panic!("unable to access storage");
    };
    JsFuture::from(storage.get_directory())
        .await
        .expect("Getting root directory failed")
        .dyn_into::<FileSystemDirectoryHandle>()
        .expect(DYN_INTO_ERROR)
}

async fn child_dir(
    parent: &FileSystemDirectoryHandle,
    name: &str,
    create: bool,
) -> Result<FileSystemDirectoryHandle> {
    let options = FileSystemGetDirectoryOptions::new();
    options.set_create(create);
    let result = JsFuture::from(parent.get_directory_handle_with_options(name, &options))
        .await
        .map_err(|e| js_value_to_error(e))?
        .dyn_into::<FileSystemDirectoryHandle>()
        .expect(DYN_INTO_ERROR);
    Ok(result)
}

async fn child_file(
    parent: &FileSystemDirectoryHandle,
    name: &str,
    create: bool,
) -> Result<FileSystemFileHandle> {
    let options = FileSystemGetFileOptions::new();
    options.set_create(create);
    let result = JsFuture::from(parent.get_file_handle_with_options(name, &options))
        .await
        .map_err(|e| js_value_to_error(e))?
        .dyn_into::<FileSystemFileHandle>()
        .expect(DYN_INTO_ERROR);
    Ok(result)
}

async fn get_parent_dir<P: AsRef<Path>>(
    path: P,
    create: bool,
) -> Result<FileSystemDirectoryHandle> {
    let path = path.as_ref();
    let root = get_root().await;
    let mut parents_stack = vec![root];
    if let Some(path) = path.parent() {
        for component in path.components() {
            match component {
                Component::Prefix(_) => return Err(Error::from(ErrorKind::PermissionDenied)),
                Component::CurDir | Component::RootDir => (),
                Component::ParentDir => {
                    if parents_stack.len() == 1 {
                        return Err(Error::from(ErrorKind::PermissionDenied));
                    } else {
                        parents_stack.pop();
                    }
                }
                Component::Normal(name) => {
                    let name = name.to_string_lossy();
                    parents_stack.push(
                        child_dir(parents_stack.last().as_ref().unwrap(), &name, create).await?,
                    );
                }
            }
        }
    }
    Ok(parents_stack.pop().unwrap())
}

async fn get_dir<P: AsRef<Path>>(
    path: P,
    create: bool,
    create_parents: bool,
) -> Result<FileSystemDirectoryHandle> {
    let parent_dir = get_parent_dir(&path, create_parents).await?;
    if let Some(name) = path.as_ref().file_name() {
        let name = name.to_string_lossy();
        child_dir(&parent_dir, &name, create).await
    } else {
        Ok(parent_dir)
    }
}

async fn get_file<P: AsRef<Path>>(path: P, create: bool) -> Result<FileSystemFileHandle> {
    let parent_dir = get_parent_dir(&path, false).await?;
    if let Some(name) = path.as_ref().file_name() {
        let name = name.to_string_lossy();
        child_file(&parent_dir, &name, create).await
    } else {
        Err(Error::from(ErrorKind::AlreadyExists))
    }
}

pub async fn create_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    get_dir(path, true, false).await?;
    Ok(())
}
pub async fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<()> {
    get_dir(path, true, true).await?;
    Ok(())
}

/// Symlink is not supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    File,
    Dir,
}
impl FileType {
    pub fn is_dir(&self) -> bool {
        *self == Self::Dir
    }
    pub fn is_file(&self) -> bool {
        *self == Self::File
    }
    pub fn is_symlink(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct DirEntry {
    name: OsString,
    file_type: FileType,
    path: PathBuf,
}
impl DirEntry {
    pub fn file_name(&self) -> OsString {
        self.name.clone()
    }
    /// Symlink is not supported. This does not actually require to async. It is async to be compatible with async-fs.
    pub async fn file_type(&self) -> Result<FileType> {
        Ok(self.file_type)
    }
    /// Currently not supported.
    pub async fn metadata(&self) -> Result<std::fs::Metadata> {
        Err(Error::other("Metadata is not supported currently"))
    }
    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }
}

pub async fn read_dir<P: AsRef<Path>>(path: P) -> Result<impl Stream<Item = Result<DirEntry>>> {
    let dir = get_dir(&path, false, false).await?;
    let stream = JsStream::from(dir.entries());
    let read_dir = stream.map(move |v| {
        let entry = v.map_err(|e| js_value_to_error(e))?;
        const RESOLVE_ENTRY_ERROR: &str =
            "Getting the key and value of the dir entry failed, this is an error of the crate.";
        let key = Reflect::get_u32(&entry, 0)
            .expect(RESOLVE_ENTRY_ERROR)
            .as_string()
            .expect("This is supposed to be a string, else this is an error of the crate.");
        let value = Reflect::get_u32(&entry, 1).expect(RESOLVE_ENTRY_ERROR);

        let mut path = path.as_ref().to_path_buf();
        path.push(&key);
        let name = OsString::from(key);
        if let Some(_) = value.dyn_ref::<FileSystemFileHandle>() {
            Ok(DirEntry {
                name,
                file_type: FileType::File,
                path,
            })
        } else {
            Ok(DirEntry {
                name,
                file_type: FileType::Dir,
                path,
            })
        }
    });
    Ok(read_dir)
}

/// Currently `remove_dir()` and `remove_file()` work the same.
pub async fn remove_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    let parent_dir = get_parent_dir(&path, false).await?;
    let name = path
        .as_ref()
        .file_name()
        .ok_or(Error::from(ErrorKind::NotFound))?
        .to_string_lossy();

    JsFuture::from(parent_dir.remove_entry(&name))
        .await
        .map_err(|e| js_value_to_error(e))?;

    Ok(())
}

/// Currently `remove_dir()` and `remove_file()` work the same.
pub async fn remove_file<P: AsRef<Path>>(path: P) -> Result<()> {
    remove_dir(path).await
}

pub async fn remove_dir_all<P: AsRef<Path>>(path: P) -> Result<()> {
    let parent_dir = get_parent_dir(&path, false).await?;
    let name = path
        .as_ref()
        .file_name()
        .ok_or(Error::from(ErrorKind::NotFound))?
        .to_string_lossy();

    let options = FileSystemRemoveOptions::new();
    options.set_recursive(true);

    JsFuture::from(parent_dir.remove_entry_with_options(&name, &options))
        .await
        .map_err(|e| js_value_to_error(e))?;

    Ok(())
}

pub async fn read<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let mut file = File::open(path).await?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).await?;
    Ok(buf)
}

pub async fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut file = File::open(path).await?;
    let mut buf = String::new();
    file.read_to_string(&mut buf).await?;
    Ok(buf)
}

pub async fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<()> {
    let mut file = File::create_new(path).await?;
    file.write_all(contents.as_ref()).await.unwrap();
    Ok(())
}

pub async fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<u64> {
    let mut src = File::open(from).await?;
    let mut dst = File::create_new(to).await?;
    let buf_size = src.size.min(1 << 6) as usize;
    let mut buf = vec![0; buf_size];
    loop {
        let read_size = src.read(&mut buf).await?;
        if read_size == 0 {
            break;
        }
        dst.write_all(&buf[0..read_size]).await?;
        buf[0..read_size].fill(0);
    }
    Ok(src.size)
}
