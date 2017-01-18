use queues::publisher::Publisher;
use queues::publisher::Subscriber;
use core::cell::UnsafeCell;
use streams::intercore::api::Message;

pub enum TypeId {
    Byte,
    Int,
    Float,
}
pub struct Ctx {
    publishers: UnsafeCell<Vec<Publisher<u64>>>,
    subscribers: UnsafeCell<Vec<Subscriber<u64>>>,
}

pub struct Channel {
    pub id: usize,
    pub publisher: Publisher<Message>,
    pub subscribers: Vec<Subscriber<Message>>,
}

impl Ctx {
    pub fn new() -> Self {
        Ctx {
            publishers: UnsafeCell::new(vec![]),
            subscribers: UnsafeCell::new(vec![]),
        }
    }
    #[inline]
    pub fn publishers(&self) -> &mut Vec<Publisher<u64>> {
        unsafe { &mut *self.publishers.get() }
    }

    #[inline]
    pub fn subscribers(&self) -> &mut Vec<Subscriber<u64>> {
        unsafe { &mut *self.subscribers.get() }
    }
}