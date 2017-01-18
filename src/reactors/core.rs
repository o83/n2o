use reactors::scheduler::{Scheduler, TaskTermination};
use streams::intercore::ctx::Channel;
use reactors::job::Job;
use queues::publisher::Subscriber;
use std::ffi::CString;
use handle::{self, Handle};
use streams::intercore::api::Message;
use queues::pubsub::PubSub;

pub struct Core<'a> {
    id: usize,
    scheduler: Scheduler<'a, Job<'a>>,
}

impl<'a> Core<'a> {
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

    pub fn park(&mut self) {
        self.scheduler.run();
    }
}

impl<'a> PubSub<Message> for Core<'a> {
    fn subscribe(&mut self) -> Subscriber<Message> {
        self.scheduler.subscribe()
    }

    fn add_subscriber(&mut self, s: Subscriber<Message>) {
        self.scheduler.add_subscriber(s);
    }
}