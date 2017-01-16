use reactors::scheduler::Scheduler;
use streams::intercore::ctx::Channel;
use reactors::job::Job;
use queues::publisher::Publisher;
use std::ffi::CString;
use handle::{self, Handle};

pub struct Core<'a> {
    scheduler: Scheduler<'a, Job<'a>>,
    bus: Handle<Channel>,
    id: usize,
}

impl<'a> Core<'a> {
    pub fn bus(&self) -> &mut Channel {
        self.bus.borrow_mut()
    }

    pub fn new(id: usize) -> Self {
        let mut subscribers = Vec::new();
        let mut publisher = Publisher::with_mirror(CString::new(format!("/core_{}", id)).unwrap(), 8);
        Core {
            id: id,
            scheduler: Scheduler::new(),
            bus: handle::new(Channel {
                publisher: publisher,
                subscribers: subscribers,
            }),
        }
    }

    pub fn connect_with(&self, other: &Self) {
        let s = self.bus().publisher.subscribe();
        self.bus().subscribers.push(s);
        let s = self.bus().publisher.subscribe();
        other.bus().subscribers.push(s);
    }

    pub fn park(&'a mut self) {
        self.scheduler.run();
    }
}