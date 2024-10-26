use std::{cell::RefCell, path::Path, rc::Rc};

use wasm_bindgen_futures::spawn_local;

use crate::{get_file, OpenFileFuture, Task, FS};

const APPEND: u8 = 0b0000_0001;
const CREATE: u8 = 0b0000_0010;
const CREATE_NEW: u8 = 0b0000_0100;
const READ: u8 = 0b0000_1000;
const TRUNCATE: u8 = 0b0001_0000;
const WRITE: u8 = 0b0010_0000;
pub struct OpenOptions(u8);

impl OpenOptions {
    pub fn new() -> Self {
        Self(0)
    }
    pub fn append(&mut self) -> &mut OpenOptions {
        self.0 |= APPEND;
        self
    }
    pub fn create(&mut self) -> &mut OpenOptions {
        self.0 |= CREATE;
        self
    }
    pub fn create_new(&mut self) -> &mut OpenOptions {
        self.0 |= CREATE_NEW;
        self
    }
    pub fn read(&mut self) -> &mut OpenOptions {
        self.0 |= READ;
        self
    }
    pub fn truncate(&mut self) -> &mut OpenOptions {
        self.0 |= TRUNCATE;
        self
    }
    pub fn write(&mut self) -> &mut OpenOptions {
        self.0 |= WRITE;
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
            let handle = get_file(path, options & CREATE > 0).await;
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
