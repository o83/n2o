use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::error::Error;
use std::fmt;

use abstractions::futures::future::Future;
use abstractions::poll::{Poll, Async};
use abstractions::queues::lock::Lock;
use abstractions::tasks::task;
use abstractions::tasks::task::Task;

#[must_use = "futures do nothing unless polled"]
pub struct Oneshot<T> {
    inner: Arc<Inner<T>>,
}


pub struct Complete<T> {
    inner: Arc<Inner<T>>,
}

struct Inner<T> {
    complete: AtomicBool,
    data: Lock<Option<T>>,
    rx_task: Lock<Option<Task>>,
    tx_task: Lock<Option<Task>>,
}

pub fn new<T>() -> (Complete<T>, Oneshot<T>) {
    let inner = Arc::new(Inner {
        complete: AtomicBool::new(false),
        data: Lock::new(None),
        rx_task: Lock::new(None),
        tx_task: Lock::new(None),
    });
    let oneshot = Oneshot { inner: inner.clone() };
    let complete = Complete { inner: inner };
    (complete, oneshot)
}

impl<T> Complete<T> {
    pub fn complete(self, t: T) {
        let mut slot = self.inner.data.try_lock().unwrap();
        assert!(slot.is_none());
        *slot = Some(t);
        drop(slot);
    }

    pub fn poll_cancel(&mut self) -> Poll<(), ()> {

        if self.inner.complete.load(SeqCst) {
            return Ok(Async::Ready(()));
        }

        let handle = task::park();
        match self.inner.tx_task.try_lock() {
            Some(mut p) => *p = Some(handle),
            None => return Ok(Async::Ready(())),
        }
        if self.inner.complete.load(SeqCst) {
            Ok(Async::Ready(()))
        } else {
            Ok(Async::NotReady)
        }
    }
}

impl<T> Drop for Complete<T> {
    fn drop(&mut self) {

        self.inner.complete.store(true, SeqCst);
        if let Some(mut slot) = self.inner.rx_task.try_lock() {
            if let Some(task) = slot.take() {
                drop(slot);
                task.unpark();
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Canceled;

impl fmt::Display for Canceled {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "oneshot canceled")
    }
}

impl Error for Canceled {
    fn description(&self) -> &str {
        "oneshot canceled"
    }
}


impl<T> Future for Oneshot<T> {
    type Item = T;
    type Error = Canceled;

    fn poll(&mut self) -> Poll<T, Canceled> {
        let mut done = false;

        if self.inner.complete.load(SeqCst) {
            done = true;
        } else {
            let task = task::park();
            match self.inner.rx_task.try_lock() {
                Some(mut slot) => *slot = Some(task),
                None => done = true,
            }
        }

        if done || self.inner.complete.load(SeqCst) {
            match self.inner.data.try_lock().unwrap().take() {
                Some(data) => Ok(data.into()),
                None => Err(Canceled),
            }
        } else {
            Ok(Async::NotReady)
        }
    }
}

impl<T> Drop for Oneshot<T> {
    fn drop(&mut self) {

        self.inner.complete.store(true, SeqCst);

        if let Some(mut slot) = self.inner.rx_task.try_lock() {
            let task = slot.take();
            drop(slot);
            drop(task);
        }

        if let Some(mut handle) = self.inner.tx_task.try_lock() {
            if let Some(task) = handle.take() {
                drop(handle);
                task.unpark()
            }
        }
    }
}
