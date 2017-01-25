use reactors::task::{self, Task, Context, Poll};
use reactors::job::Job;
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
    pub queues: Rc<Ctx>,
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
            queues: Rc::new(Ctx::new()),
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
    fn poll_bus(&'a mut self) {
        let mut bus = with(self, |h| &h.bus);
        for s in &bus.subscribers {
            handle_intercore(self, s.recv(), &mut bus);
            s.commit()
        }

    }

    pub fn run(&mut self) -> Poll<Context<'a>, task::Error> {
        let h = into_raw(self);
        let mut res: Poll<Context<'a>, task::Error> = Poll::End(Context::Nil);

        // this should be totally rewritten

        let mut bus = with(from_raw(h), |x| &x.bus);

        'start: loop {

            for s in &bus.subscribers {
                handle_intercore(self, s.recv(), &mut bus);
                s.commit()
            }

            thread::sleep(time::Duration::from_millis(100));

            for (i, t) in from_raw(h).tasks.iter_mut().enumerate() {
                let c = from_raw(h).ctxs.get_mut(i).expect("Scheduler: can't retrieve a ctx.");
                let (a, b) = split(&mut t.0);
                let y = a.poll(Context::Nil);
                match y {
                    Poll::Yield(Context::Intercore(ref m)) => {
                        let ha = handle_intercore(self, Some(m), &mut bus);
                        println!("IC: {:?}", m);
                        continue 'start;
                    }
                    Poll::End(v) => {
                        println!("End: {:?}", v);
                        from_raw(h).terminate(t.1, i);
                        return Poll::End(v);
                    }
                    Poll::Err(e) => {
                        println!("Err: {:?}", e);
                        from_raw(h).terminate(t.1, i);
                        return Poll::Err(e);
                    }
                    z => {
                        println!("X: {:?}", z);
                        res = z;
                    }
                }
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
