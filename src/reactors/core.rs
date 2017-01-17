use reactors::scheduler::Scheduler;
use streams::intercore::ctx::Channel;
use reactors::job::Job;
use queues::publisher::Publisher;
use std::ffi::CString;
use handle::{self, Handle};
use streams::intercore::api::Message;

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

    pub fn connect_with(&'a self, other: &'a Self) {
        let s = self.bus().publisher.subscribe();
        self.bus().subscribers.push(s);
        let s = self.bus().publisher.subscribe();
        other.bus().subscribers.push(s);
    }

    pub fn park(&mut self) {
        self.scheduler.run();
    }

    pub fn publish<F, R>(&mut self, mut f: F) -> R
        where F: FnMut(&mut Publisher<Message>) -> R
    {
        f(&mut self.bus.borrow_mut().publisher)
    }
}