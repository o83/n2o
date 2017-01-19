use std::rc::Rc;
use streams::intercore::ctx::Ctx;
use reactors::system::IO;
use reactors::selector::{Selector, Async, Pool};
use reactors::scheduler::{Scheduler, TaskTermination, TaskId};
use reactors::job::Job;
use streams::intercore::api::Message;
use queues::publisher::{Publisher, Subscriber};
use reactors::cpstask::CpsTask;
use queues::pubsub::PubSub;
// use reactors::ws::WsServer;
// use reactors::console::Console;
// use std::net::SocketAddr;
use std::ffi::CString;
use handle;
use std::str;

pub struct Boot<'a> {
    io: IO,
    scheduler: Scheduler<'a>,
    publisher: Publisher<Message>,
}

impl<'a> Boot<'a> {
    pub fn new() -> Self {
        Boot {
            io: IO::new(),
            scheduler: Scheduler::new(),
            publisher: Publisher::with_mirror(CString::new(format!("/boot_{}", 0)).unwrap(), 8),
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
        s1.scheduler.exec(t, Some(x));
        let r = s2.scheduler.run();
        // check and handle builtin
        s2.io.write_all(format!("{:?}\n", r).as_bytes());
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
        let h = handle::into_raw(self);
        let j = Job::Cps(CpsTask::new(Rc::new(Ctx::new())));
        let task_id = handle::from_raw(h).scheduler.spawn(j, TaskTermination::Corecursive, None);
        loop {
            match handle::from_raw(h).io.poll() {
                Async::Ready((_, p)) => handle::from_raw(h).ready(p, task_id),
                Async::NotReady => (),
            }
        }
    }

    pub fn publish<F, R>(&mut self, mut f: F) -> R
        where F: FnMut(&mut Publisher<Message>) -> R
    {
        f(&mut self.publisher)
    }
}

impl<'a> PubSub<Message> for Boot<'a> {
    fn subscribe(&mut self) -> Subscriber<Message> {
        self.publisher.subscribe()
    }

    fn add_subscriber(&mut self, s: Subscriber<Message>) {
        unimplemented!();
    }
}