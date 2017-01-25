use std::rc::Rc;
use std::ffi::CString;
use std::str;
use reactors::system::IO;
use reactors::selector::{Selector, Async, Pool};
use reactors::scheduler::{Scheduler, TaskTermination, TaskId};
use reactors::job::Job;
use reactors::cpstask::CpsTask;
use intercore::message::Message;
use intercore::bus::{Channel, Ctx};
use queues::publisher::{Publisher, Subscriber};
use queues::pubsub::PubSub;
use handle::{self, into_raw, from_raw, with};

pub struct Boot<'a> {
    io: IO,
    scheduler: Scheduler<'a>,
}

impl<'a> Boot<'a> {
    pub fn new() -> Self {
        Boot {
            io: IO::new(),
            scheduler: Scheduler::with_channel(0),
        }
    }

    pub fn add_selected(&mut self, s: Selector) {
        self.io.spawn(s);
    }

    #[inline]
    fn handle_builtin(&'a mut self, t: TaskId, b: &'a [u8]) {
        //
    }

    #[inline]
    fn handle_raw(&'a mut self, t: TaskId, b: &'a [u8]) {
        if b.len() == 0 {
            return;
        }
        if b.len() == 1 && b[0] == 0x0A {
            self.io.write_all(&[0u8; 0]);
            return;
        }
        let x = str::from_utf8(b).unwrap();
        let (s1, s2) = handle::split(self);
        let (s3, s4) = handle::split(s2);
        let (s5, s6) = handle::split(s4);

        s1.scheduler.exec(t, Some(x));
        let r = s5.scheduler.run();
        s6.io.write_all(format!("{:?}\n", r).as_bytes());
    }

    #[inline]
    fn handle_msg(&'a mut self, t: TaskId, b: &[Message]) {
        for msg in b {
            match *msg {
                ref x => println!("Unsupported intercore Message: {:?}", x),
            }
        }
    }

    #[inline]
    fn ready(&'a mut self, p: Pool<'a>, t: TaskId) {
        match p {
            Pool::Raw(b) => self.handle_raw(t, b),
            Pool::Msg(b) => self.handle_msg(t, b.buf),
        }
    }

    pub fn init(&mut self) {
        let h = into_raw(self);
        let j = Job::Cps(CpsTask::new());
        let task_id = from_raw(h).scheduler.spawn(j, TaskTermination::Corecursive, None);
        loop {
            match from_raw(h).io.poll() {
                Async::Ready((_, p)) => from_raw(h).ready(p, task_id),
                Async::NotReady => (),
            }
        }
    }
}

impl<'a> PubSub<Message> for Boot<'a> {
    fn subscribe(&mut self) -> Subscriber<Message> {
        self.scheduler.bus.publisher.subscribe()
    }

    fn add_subscriber(&mut self, s: Subscriber<Message>) {
        unimplemented!();
    }
}