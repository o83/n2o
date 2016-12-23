use queues::publisher::Publisher;
use queues::publisher::Subscriber;
use std::cell::UnsafeCell;

pub struct Ctx<T> {
    publishers: UnsafeCell<Vec<Publisher<T>>>,
    subscribers: UnsafeCell<Vec<Subscriber<T>>>,
}

impl<T> Ctx<T> {
    pub fn new() -> Self {
        Ctx {
            publishers: UnsafeCell::new(vec![]),
            subscribers: UnsafeCell::new(vec![]),
        }
    }
    #[inline]
    pub fn publishers(&self) -> &mut Vec<Publisher<T>> {
        unsafe { &mut *self.publishers.get() }
    }

    #[inline]
    pub fn subscribers(&self) -> &mut Vec<Subscriber<T>> {
        unsafe { &mut *self.subscribers.get() }
    }
}

pub struct Ctxs<T> {
    ctxs: Vec<Ctx<T>>,
}

impl<T> Ctxs<T> {
    pub fn new() -> Self {
        Ctxs { ctxs: vec![] }
    }
}