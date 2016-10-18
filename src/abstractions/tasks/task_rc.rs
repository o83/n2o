use std::prelude::v1::*;
use std::sync::Arc;
use std::cell::UnsafeCell;
use abstractions::tasks::task;

pub struct TaskRc<A> {
    task_id: usize,
    ptr: Arc<UnsafeCell<A>>,
}

unsafe impl<A: Send> Send for TaskRc<A> {}
unsafe impl<A: Sync> Sync for TaskRc<A> {}

impl<A> TaskRc<A> {
    pub fn new(a: A) -> TaskRc<A> {
        task::with(|task, _| {
            TaskRc {
                task_id: task.id,
                ptr: Arc::new(UnsafeCell::new(a)),
            }
        })
    }

    pub fn with<F, R>(&self, f: F) -> R
        where F: FnOnce(&A) -> R
    {
        task::with(|task, _| {
            assert!(self.task_id == task.id,
                    "TaskRc being accessed on task it does not belong to");
            f(unsafe { &*self.ptr.get() })
        })
    }
}

impl<A> Clone for TaskRc<A> {
    fn clone(&self) -> TaskRc<A> {
        TaskRc {
            task_id: self.task_id,
            ptr: self.ptr.clone(),
        }
    }
}
