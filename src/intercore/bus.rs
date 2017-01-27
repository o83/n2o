use queues::publisher::Publisher;
use queues::publisher::Subscriber;
use core::cell::UnsafeCell;
use intercore::message::Message;

pub fn send<'a>(bus: &'a Channel, m: Message) {
    if let Some(v) = bus.publisher.next() {
        *v = m;
        bus.publisher.commit();
    };
}

pub enum TypeId {
    Byte,
    Int,
    Float,
}

pub struct Memory {
    publishers: UnsafeCell<Vec<Publisher<i64>>>,
    subscribers: UnsafeCell<Vec<Subscriber<i64>>>,
}

pub struct Channel {
    pub id: usize,
    pub publisher: Publisher<Message>,
    pub subscribers: Vec<Subscriber<Message>>,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            publishers: UnsafeCell::new(vec![]),
            subscribers: UnsafeCell::new(vec![]),
        }
    }
    #[inline]
    pub fn publishers(&self) -> &mut Vec<Publisher<i64>> {
        unsafe { &mut *self.publishers.get() }
    }

    #[inline]
    pub fn subscribers(&self) -> &mut Vec<Subscriber<i64>> {
        unsafe { &mut *self.subscribers.get() }
    }
}