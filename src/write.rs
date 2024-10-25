use std::{
    cell::RefCell,
    io::Result,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use futures_lite::AsyncWrite;
use js_sys::{Object, Uint8Array};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::JsValue;

use crate::{
    set_value, File, Fs, OutMsg, Task, BUF, FD, FS, INDEX, POST_ERROR, TO_VALUE_ERROR, WRITE,
};

impl Fs {
    fn write(&self, fd: usize, buf: &[u8], task: Rc<RefCell<Task<Result<usize>>>>) {
        let write_obj = Object::new();
        let index = self.inner.borrow_mut().writing_tasks.insert(task);
        let typed_array = Uint8Array::new_with_length(buf.len() as u32);
        typed_array.copy_from(buf);

        set_value(&write_obj, &INDEX, &JsValue::from(index));
        set_value(&write_obj, &FD, &JsValue::from(fd));
        set_value(&write_obj, &BUF, &typed_array.buffer());
        let msg = Object::new();
        set_value(&msg, &WRITE, &write_obj);

        self.worker.post_message(&msg).expect(POST_ERROR);
    }
    fn flush(&self, fd: usize, task: Rc<RefCell<Task<Result<()>>>>) {
        let index = self.inner.borrow_mut().flushing_tasks.insert(task);
        self.worker
            .post_message(&to_value(&OutMsg::Flush { fd, index }).expect(TO_VALUE_ERROR))
            .expect(POST_ERROR);
    }
    fn close(&self, fd: usize, task: Rc<RefCell<Task<Result<()>>>>) {
        let index = self.inner.borrow_mut().closing_tasks.insert(task);
        self.worker
            .post_message(&to_value(&OutMsg::Close { fd, index }).expect(TO_VALUE_ERROR))
            .expect(POST_ERROR);
    }
}

impl AsyncWrite for File {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
        let task = if let Some(task) = self.write_task.clone() {
            task
        } else {
            let task = Rc::new(RefCell::new(Task {
                waker: Some(cx.waker().clone()),
                result: None,
            }));
            let task_clone = task.clone();
            FS.with_borrow(|fs| fs.write(self.fd, buf, task_clone));

            self.write_task = Some(task.clone());
            task
        };
        let mut inner = task.borrow_mut();
        if let Some(result) = inner.result.take() {
            Poll::Ready(result)
        } else {
            Poll::Pending
        }
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        let task = if let Some(task) = self.flush_task.clone() {
            task
        } else {
            let task = Rc::new(RefCell::new(Task {
                waker: Some(cx.waker().clone()),
                result: None,
            }));
            let task_clone = task.clone();
            FS.with_borrow(|fs| fs.flush(self.fd, task_clone));

            self.flush_task = Some(task.clone());
            task
        };
        let mut inner = task.borrow_mut();
        if let Some(result) = inner.result.take() {
            Poll::Ready(result)
        } else {
            Poll::Pending
        }
    }
    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        let task = if let Some(task) = self.close_task.clone() {
            task
        } else {
            let task = Rc::new(RefCell::new(Task {
                waker: Some(cx.waker().clone()),
                result: None,
            }));
            let task_clone = task.clone();
            FS.with_borrow(|fs| fs.close(self.fd, task_clone));

            self.close_task = Some(task.clone());
            task
        };
        let mut inner = task.borrow_mut();
        if let Some(result) = inner.result.take() {
            Poll::Ready(result)
        } else {
            Poll::Pending
        }
    }
}
