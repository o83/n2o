use reactors::task::{self, Task, Context, Poll};
use reactors::job::Job;
use reactors::system::IO;
use reactors::cpstask::CpsTask;
use intercore::message::*;
use intercore::bus::{Ctx, Channel, send};
use intercore::server::handle_intercore;
use queues::publisher::{Publisher, Subscriber};
use queues::pubsub::PubSub;
use commands::ast::{AST, Value};
use std::rc::Rc;
use std::mem;
use std::{thread, time};
use std::ffi::CString;
use handle::{self, from_raw, into_raw, with, split};
use reactors::console::Console;
use reactors::selector::{Selector, Async, Pool};

const TASKS_MAX_CNT: usize = 256;

#[derive(Debug,Clone,Copy)]
pub struct TaskId(usize);

#[derive(Debug,PartialEq,Clone,Copy)]
pub enum TaskTermination {
    Recursive,
    Corecursive,
}

#[derive(Debug)]
pub struct T3<T>(pub T, pub TaskTermination);

pub struct Scheduler<'a> {
    pub tasks: Vec<T3<Job<'a>>>,
    pub ctxs: Vec<Context<'a>>,
    pub bus: Channel,
    pub io: IO,
    pub queues: Ctx,
}

impl<'a> Scheduler<'a> {
    pub fn with_channel(id: usize) -> Self {
        let chan = Channel {
            id: id,
            publisher: Publisher::with_mirror(CString::new(format!("/pub_{}", id)).unwrap(), 8),
            subscribers: Vec::new(),
        };
        Scheduler {
            tasks: Vec::with_capacity(TASKS_MAX_CNT),
            ctxs: Vec::with_capacity(TASKS_MAX_CNT),
            bus: chan,
            io: IO::new(),
            queues: Ctx::new(),
        }
    }

    pub fn spawn(&'a mut self, t: Job<'a>, l: TaskTermination, input: Option<&'a str>) -> TaskId {
        let last = self.tasks.len();
        self.tasks.push(T3(t, l));
        self.ctxs.push(Context::Nil);
        self.tasks.last_mut().expect("Scheduler: can't retrieve a task.").0.init(input, last);
        TaskId(last)
    }

    pub fn exec(&'a mut self, t: TaskId, input: Option<&'a str>) {
        self.tasks.get_mut(t.0).expect("Scheduler: can't retrieve a task.").0.exec(input);
    }

    #[inline]
    fn terminate(&'a mut self, t: TaskTermination, i: usize) {
        if t == TaskTermination::Recursive {
            self.tasks.remove(i);
            self.ctxs.remove(i);
        }
    }

    #[inline]
    fn poll_bus(&mut self) {
        let mut bus = with(self, |h| &h.bus);
        for s in &bus.subscribers {
            handle_intercore(self, s.recv(), &mut bus);
            s.commit()
        }

    }

    pub fn run(&mut self) -> Poll<Context<'a>, task::Error> {
        let res: Poll<Context<'a>, task::Error> = Poll::End(Context::Nil);
        loop {
            self.poll_bus();
        }
        res
    }

    pub fn run0(&mut self) -> Poll<Context<'a>, task::Error> {
        println!("bsp_run...");
        let res: Poll<Context<'a>, task::Error> = Poll::End(Context::Nil);
        let x = into_raw(self);
        from_raw(x).io = IO::new();
        from_raw(x).io.spawn(Selector::Rx(Console::new()));
        loop {
            from_raw(x).poll_bus();
            match from_raw(x).io.poll() {
                Async::Ready((_, Pool::Raw(buf))) => {
                   println!("Raw: {:?}", buf);
                   send(&from_raw(x).bus, Message::Pub(Pub {
                         from: 0,
                         task_id: 0,
                         to: 1,
                         name: "".to_string(),
                         cap: 8, }));
                }
                Async::Ready((_, _)) => (),
                Async::NotReady => (),
            }
        }
        res
    }
}

impl<'a> PubSub<Message> for Scheduler<'a> {
    fn subscribe(&mut self) -> Subscriber<Message> {
        self.bus.publisher.subscribe()
    }

    fn add_subscriber(&mut self, s: Subscriber<Message>) {
        self.bus.subscribers.push(s);
    }
}
