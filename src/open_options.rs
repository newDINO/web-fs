use std::{
    cell::RefCell,
    future::Future,
    io::Result,
    path::Path,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use js_sys::Object;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::FileSystemFileHandle;

use crate::{get_file, set_value, File, Fs, Task, FS, HANDLE, INDEX, OPEN, OPTIONS, POST_ERROR};

const APPEND: u8 = 0b0000_0001;
const CREATE: u8 = 0b0000_0010;
const CREATE_NEW: u8 = 0b0000_0100;
const READ: u8 = 0b0000_1000;
const TRUNCATE: u8 = 0b0001_0000;
const WRITE: u8 = 0b0010_0000;
pub struct OpenOptions(u8);

impl Fs {
    fn open(
        &self,
        handle: FileSystemFileHandle,
        options: u8,
        inner: Rc<RefCell<Task<Result<File>>>>,
    ) {
        let index = self.inner.borrow_mut().opening_tasks.insert(inner);

        let open = Object::new();
        set_value(&open, &INDEX, &JsValue::from(index));
        set_value(&open, &HANDLE, &handle);
        set_value(&open, &OPTIONS, &JsValue::from(options));
        let msg = Object::new();
        set_value(&msg, &OPEN, &open);

        self.worker.post_message(&msg).expect(POST_ERROR);
    }
}

pub struct OpenFileFuture {
    inner: Rc<RefCell<Task<Result<File>>>>,
    append: bool,
}
impl Future for OpenFileFuture {
    type Output = Result<File>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut inner = self.inner.borrow_mut();

        if let Some(val) = inner.result.take() {
            let result = val.map(|mut file| {
                if self.append {
                    file.cursor = file.size
                }
                file
            });
            return Poll::Ready(result);
        }
        inner.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}
impl OpenOptions {
    pub fn new() -> Self {
        Self(0)
    }
    fn set_bit(&mut self, bit: u8, value: bool) {
        if value {
            self.0 |= bit;
        } else {
            self.0 &= !bit;
        }
    }
    pub fn append(&mut self, append: bool) -> &mut OpenOptions {
        self.set_bit(APPEND, append);
        self
    }
    pub fn create(&mut self, create: bool) -> &mut OpenOptions {
        self.set_bit(CREATE, create);
        self
    }
    pub fn create_new(&mut self, create_new: bool) -> &mut OpenOptions {
        self.set_bit(CREATE_NEW, create_new);
        self
    }
    pub fn read(&mut self, read: bool) -> &mut OpenOptions {
        self.set_bit(READ, read);
        self
    }
    pub fn truncate(&mut self, truncate: bool) -> &mut OpenOptions {
        self.set_bit(TRUNCATE, truncate);
        self
    }
    pub fn write(&mut self, write: bool) -> &mut OpenOptions {
        self.set_bit(WRITE, write);
        self
    }
    pub fn open<P: AsRef<Path>>(&self, path: P) -> OpenFileFuture {
        let path = path.as_ref().to_string_lossy().to_string();

        let state = Task {
            waker: None,
            result: None,
        };
        let inner = Rc::new(RefCell::new(state));
        let inner_clone = inner.clone();

        let options = self.0;
        spawn_local(async move {
            let handle = get_file(path, options & CREATE | options & CREATE_NEW > 0).await;
            match handle {
                Ok(handle) => FS.with_borrow(|fs| fs.open(handle, options, inner_clone)),
                Err(e) => {
                    let mut state = inner_clone.borrow_mut();
                    state.result = Some(Err(e));
                    if let Some(waker) = state.waker.take() {
                        waker.wake();
                    }
                }
            }
        });
        OpenFileFuture {
            inner,
            append: self.0 & APPEND > 0,
        }
    }
}
