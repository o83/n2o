use reactors::task::{self, Task, Context, Poll};
use reactors::job::Job;
use reactors::system::IO;
use reactors::cpstask::CpsTask;
use intercore::message::*;
use intercore::bus::{Ctx, Channel, send};
use intercore::server::handle_intercore;
use queues::publisher::{Publisher, Subscriber};
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
            publisher: Publisher::with_mirror(CString::new(format!("/pub_{}", id)).unwrap(), 88),
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
        let x = into_raw(self);
        for s in &from_raw(x).bus.subscribers {
            handle_intercore(self, s.recv(), &mut from_raw(x).bus, s);
            s.commit();
        }
    }

    pub fn handle_message(&mut self, buf: &'a [u8]) {
        let bus = self.bus.id;
        println!("Message on REPL bus ({:?}): {:?}", bus, buf);
        send(&self.bus,
             Message::Pub(Pub {
                 from: bus,
                 task_id: 0,
                 to: 1,
                 name: "".to_string(),
                 cap: 8,
             }));
    }

    pub fn hibernate(&mut self) {
        thread::sleep(time::Duration::from_millis(10)); // Green Peace
    }

    pub fn run0(&mut self) {
        println!("BSP run on core {:?}", self.bus.id);
        self.io.spawn(Selector::Rx(Console::new()));
        let x = into_raw(self);
        loop {
            self.poll_bus();
            match from_raw(x).io.poll() {
                Async::Ready((_, Pool::Raw(buf))) => self.handle_message(buf),
                _ => (),
            }
            self.hibernate();
        }
    }

    pub fn run(&mut self) {
        println!("AP run on core {:?}", self.bus.id);
        loop {
            self.poll_bus();
            self.hibernate();
        }
    }
}
