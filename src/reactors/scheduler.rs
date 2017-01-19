use reactors::task::{self, Task, Context, Poll};
use reactors::job::Job;
use reactors::cpstask::CpsTask;
use streams::intercore::ctx::Channel;
use queues::publisher::Subscriber;
use queues::pubsub::PubSub;
use streams::intercore::api::Message;
use streams::intercore::ctx::Ctx;
use std::rc::Rc;
use std::mem;
use handle;

const TASKS_MAX_CNT: usize = 256;

#[derive(Debug,Clone,Copy)]
pub struct TaskId(usize);

#[derive(Debug,PartialEq,Clone,Copy)]
pub enum TaskTermination {
    Recursive,
    Corecursive,
}

#[derive(Debug)]
struct T3<T>(T, TaskTermination);

pub struct Scheduler<'a> {
    tasks: Vec<T3<Job<'a>>>,
    ctxs: Vec<Context<'a>>,
    bus: Option<Channel>,
}

impl<'a> Scheduler<'a> {
    pub fn new() -> Self {
        Scheduler {
            tasks: Vec::with_capacity(TASKS_MAX_CNT),
            ctxs: Vec::with_capacity(TASKS_MAX_CNT),
            bus: None,
        }
    }

    pub fn set_channel(mut self, c: Channel) -> Self {
        self.bus = Some(c);
        self
    }

    pub fn spawn(&'a mut self, t: Job<'a>, l: TaskTermination, input: Option<&'a str>) -> TaskId {
        let last = self.tasks.len();
        self.tasks.push(T3(t, l));
        self.ctxs.push(Context::Nil);
        self.tasks.last_mut().expect("Scheduler: can't retrieve a task.").0.init(input);
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
        if let Some(ref bus) = handle::with(self, |h| h.bus.as_ref()) {
            for s in &bus.subscribers {
                match s.recv() {
                    Some(v) => {
                        println!("poll bus on core_{} {:?}", bus.id, v);
                        handle::with(self, |h| {
                            h.spawn(Job::Cps(CpsTask::new(Rc::new(Ctx::new()))),
                                    TaskTermination::Recursive,
                                    None)
                        });
                        s.commit();
                    }
                    None => {}
                }
            }
        }
    }

    pub fn run(&mut self) -> Poll<Context<'a>, task::Error> {
        let h = handle::into_raw(self);
        loop {
            handle::from_raw(h).poll_bus();
            for (i, t) in handle::from_raw(h).tasks.iter_mut().enumerate() {
                let c = handle::from_raw(h).ctxs.get_mut(i).expect("Scheduler: can't retrieve a ctx.");
                let mut ctx = mem::replace(c, Context::Nil);
                match t.0.poll(ctx) {
                    Poll::Yield(..) => (),
                    Poll::End(v) => {
                        handle::from_raw(h).terminate(t.1, i);
                        return Poll::End(v);
                    }
                    Poll::Err(e) => {
                        handle::from_raw(h).terminate(t.1, i);
                        return Poll::Err(e);
                    }
                }
            }
        }
    }
}

impl<'a> PubSub<Message> for Scheduler<'a> {
    fn subscribe(&mut self) -> Subscriber<Message> {
        self.bus.as_mut().expect("This scheduler without bus!").publisher.subscribe()
    }

    fn add_subscriber(&mut self, s: Subscriber<Message>) {
        self.bus.as_mut().expect("This scheduler without bus!").subscribers.push(s);
    }
}
