use std::{
    cell::RefCell,
    io::Result,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use futures_lite::AsyncRead;
use js_sys::{ArrayBuffer, Uint8Array};
use serde_wasm_bindgen::to_value;

use crate::{File, Fs, OutMsg, Task, FS};

pub(crate) struct ReadResult {
    pub buf: ArrayBuffer,
    pub size: usize,
}

impl Fs {
    fn read(&self, fd: usize, size: usize, task: Rc<RefCell<Task<Result<ReadResult>>>>) {
        let index = self.inner.borrow_mut().reading_tasks.insert(task);

        self.worker
            .post_message(&to_value(&OutMsg::Read { fd, size, index }).unwrap())
            .unwrap();
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
            FS.with_borrow(|fs| fs.read(self.fd, buf.len(), task_clone));

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
            Poll::Ready(Ok(result.size))
        } else {
            Poll::Pending
        }
    }
}
