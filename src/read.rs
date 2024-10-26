use std::{
    cell::RefCell,
    io::Result,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use futures_lite::AsyncRead;
use js_sys::{ArrayBuffer, Object, Uint8Array};
use wasm_bindgen::JsValue;

use crate::{set_value, File, Fs, Task, CURSOR, FD, FS, INDEX, READ, SIZE};

pub(crate) struct ReadResult {
    pub buf: ArrayBuffer,
    pub size: usize,
}

impl Fs {
    fn read(&self, fd: usize, size: usize, cursor: u64, task: Rc<RefCell<Task<Result<ReadResult>>>>) {
        let index = self.inner.borrow_mut().reading_tasks.insert(task);

        let msg = Object::new();
        let read = Object::new();
        set_value(&read, &FD, &JsValue::from(fd));
        set_value(&read, &SIZE, &JsValue::from(size));
        set_value(&read, &INDEX, &JsValue::from(index));
        set_value(&read, &CURSOR, &JsValue::from(cursor));
        set_value(&msg, &READ, &read);

        self.worker.post_message(&msg).unwrap()
    }
}

impl AsyncRead for File {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        let task = if let Some(task) = self.read_task.clone() {
            task
        } else {
            let task = Rc::new(RefCell::new(Task {
                waker: Some(cx.waker().clone()),
                result: None,
            }));
            let task_clone = task.clone();
            FS.with_borrow(|fs| fs.read(self.fd, buf.len(), self.cursor, task_clone));

            self.read_task = Some(task.clone());
            task
        };
        let mut inner = task.borrow_mut();
        if let Some(result) = inner.result.take() {
            let result = result?;
            let array = Uint8Array::new(&result.buf);
            array
                .slice(0, result.size as u32)
                .copy_to(&mut buf[..result.size]);
            self.read_task = None;
            self.cursor += result.size as u64;
            Poll::Ready(Ok(result.size))
        } else {
            Poll::Pending
        }
    }
}
