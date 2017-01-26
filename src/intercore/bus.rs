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
    publishers: UnsafeCell<Vec<Publisher<Message>>>,
    subscribers: UnsafeCell<Vec<Subscriber<Message>>>,
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
    pub fn publishers(&self) -> &mut Vec<Publisher<Message>> {
        unsafe { &mut *self.publishers.get() }
    }

    #[inline]
    pub fn subscribers(&self) -> &mut Vec<Subscriber<Message>> {
        unsafe { &mut *self.subscribers.get() }
    }
}