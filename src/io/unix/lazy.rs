// #
//
// lazy.rs
// Copyright (C) 2016 Lynx ltd. <anton@algotradinghub.com>
// Created by Anton Kundenko.
//

use std::cell::Cell;
use std::ptr;
use std::sync::Arc;
use super::mutex::Mutex;
use alloc::boxed::FnBox;

type Queue = Vec<Box<FnBox()>>;

static LOCK: Mutex = Mutex::new();
static mut QUEUE: *mut Queue = ptr::null_mut();

unsafe fn init() -> bool {
    if QUEUE.is_null() {
        let state: Box<Queue> = box Vec::new();
        QUEUE = Box::into_raw(state);
    } else if QUEUE as usize == 1 {
        // can't re-init after a cleanup
        return false;
    }

    true
}

pub fn push(f: Box<FnBox()>) -> bool {
    let mut ret = true;
    unsafe {
        LOCK.lock();
        if init() {
            (*QUEUE).push(f);
        } else {
            ret = false;
        }
        LOCK.unlock();
    }
    ret
}

pub fn at_exit<F: FnOnce() + Send + 'static>(f: F) -> Result<(), ()> {
    if push(Box::new(f)) { Ok(()) } else { Err(()) }
}

pub struct Lazy<T> {
    lock: Mutex,
    ptr: Cell<*mut Arc<T>>,
    init: fn() -> Arc<T>,
}

unsafe impl<T> Sync for Lazy<T> {}

impl<T: Send + Sync + 'static> Lazy<T> {
    pub const fn new(init: fn() -> Arc<T>) -> Lazy<T> {
        Lazy {
            lock: Mutex::new(),
            ptr: Cell::new(ptr::null_mut()),
            init: init,
        }
    }

    pub fn get(&'static self) -> Option<Arc<T>> {
        unsafe {
            self.lock.lock();
            let ptr = self.ptr.get();
            let ret = if ptr.is_null() {
                Some(self.init())
            } else if ptr as usize == 1 {
                None
            } else {
                Some((*ptr).clone())
            };
            self.lock.unlock();
            return ret;
        }
    }

    unsafe fn init(&'static self) -> Arc<T> {
        // If we successfully register an at exit handler, then we cache the
        // `Arc` allocation in our own internal box (it will get deallocated by
        // the at exit handler). Otherwise we just return the freshly allocated
        // `Arc`.
        let registered = at_exit(move || {
            self.lock.lock();
            let ptr = self.ptr.get();
            self.ptr.set(1 as *mut _);
            self.lock.unlock();
            drop(Box::from_raw(ptr))
        });
        let ret = (self.init)();
        if registered.is_ok() {
            self.ptr.set(Box::into_raw(Box::new(ret.clone())));
        }
        ret
    }
}
