use reactors::scheduler::Scheduler;
use streams::intercore::ctx::Channel;
use reactors::job::Job;
use std::cell::UnsafeCell;
use queues::publisher::Publisher;
use std::ffi::CString;

pub struct Core<'a> {
    scheduler: Scheduler<'a, Job<'a>>,
    bus: UnsafeCell<Channel>,
    id: usize,
}

impl<'a> Core<'a> {
    pub fn new(id: usize) -> Self {
        let mut subscribers = Vec::new();
        let mut publisher = Publisher::with_mirror(CString::new(format!("/core_{}", id)).unwrap(), 8);
        Core {
            id: id,
            scheduler: Scheduler::new(),
            bus: UnsafeCell::new(Channel {
                publisher: publisher,
                subscribers: subscribers,
            }),
        }
    }

    pub fn connect_with(&self, other: &Self) {
        let s = unsafe { (&mut *other.bus.get()).publisher.subscribe() };
        unsafe { (&mut *self.bus.get()).subscribers.push(s) };
        let s = unsafe { (&mut *self.bus.get()).publisher.subscribe() };
        unsafe { (&mut *other.bus.get()).subscribers.push(s) };
    }
}