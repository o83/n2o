use reactors::scheduler::{Scheduler, TaskTermination};
use streams::intercore::ctx::Channel;
use reactors::job::Job;
use queues::publisher::Publisher;
use std::ffi::CString;
use handle::{self, Handle};
use streams::intercore::api::Message;

pub struct Core<'a> {
    id: usize,
    scheduler: Scheduler<'a, Job<'a>>,
}

impl<'a> Core<'a> {
    pub fn bus(&self) -> &mut Channel {
        self.bus.borrow_mut()
    }

    pub fn new(id: usize) -> Self {
        Core {
            id: id,
            scheduler: Scheduler::new(),
        }
    }

    pub fn with_channel(id: usize, c: Channel) -> Self {
        Core {
            id: id,
            scheduler: Scheduler::with_channel(c),
        }
    }

    pub fn connect_with(&'a self, other: &'a Self) {
        // let s = self.bus().publisher.subscribe();
        // self.bus().subscribers.push(s);
        // let s = self.bus().publisher.subscribe();
        // other.bus().subscribers.push(s);
    }

    pub fn park(&mut self) {
        self.scheduler.run();
    }
}