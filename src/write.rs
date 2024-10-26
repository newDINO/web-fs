use std::{
    cell::RefCell,
    io::Result,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use futures_lite::AsyncWrite;
use js_sys::{Object, Uint8Array};
use wasm_bindgen::JsValue;

use crate::{
    set_value, File, Fs, Task, BUF, CLOSE, CURSOR, FD, FLUSH, FS, INDEX, POST_ERROR, WRITE,
};

impl Fs {
    fn write(&self, fd: usize, buf: &[u8], cursor: u64, task: Rc<RefCell<Task<Result<usize>>>>) {
        let write_obj = Object::new();
        let index = self.inner.borrow_mut().writing_tasks.insert(task);
        let typed_array = Uint8Array::new_with_length(buf.len() as u32);
        typed_array.copy_from(buf);

        set_value(&write_obj, &INDEX, &JsValue::from(index));
        set_value(&write_obj, &FD, &JsValue::from(fd));
        set_value(&write_obj, &BUF, &typed_array.buffer());
        set_value(&write_obj, &CURSOR, &JsValue::from_f64(cursor as f64));
        let msg = Object::new();
        set_value(&msg, &WRITE, &write_obj);

        self.worker.post_message(&msg).expect(POST_ERROR);
    }
    fn flush(&self, fd: usize, task: Rc<RefCell<Task<Result<()>>>>) {
        let index = self.inner.borrow_mut().flushing_tasks.insert(task);

        let msg = Object::new();
        let flust = Object::new();
        set_value(&flust, &FD, &JsValue::from(fd));
        set_value(&flust, &INDEX, &JsValue::from(index));
        set_value(&msg, &FLUSH, &flust);

        self.worker.post_message(&msg).expect(POST_ERROR);
    }
    fn close(&self, fd: usize, task: Rc<RefCell<Task<Result<()>>>>) {
        let index = self.inner.borrow_mut().closing_tasks.insert(task);

        let msg = Object::new();
        let close = Object::new();
        set_value(&close, &FD, &JsValue::from(fd));
        set_value(&close, &INDEX, &JsValue::from(index));
        set_value(&msg, &CLOSE, &close);

        self.worker.post_message(&msg).expect(POST_ERROR);
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
            FS.with_borrow(|fs| fs.write(self.fd, buf, self.cursor, task_clone));

            self.write_task = Some(task.clone());
            task
        };
        let mut inner = task.borrow_mut();
        if let Some(result) = inner.result.take() {
            self.write_task = None;
            if let Ok(size) = result {
                self.size += size as u64;
            }
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
            self.flush_task = None;
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
            self.close_task = None;
            Poll::Ready(result)
        } else {
            Poll::Pending
        }
    }
}
