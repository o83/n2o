use reactors::task::{self, Task, Context, Poll};
use streams::intercore::ctx::Channel;
use queues::publisher::Subscriber;
use queues::pubsub::PubSub;
use streams::intercore::api::Message;
use std::mem;

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

pub struct Scheduler<'a, T: 'a> {
    tasks: Vec<T3<T>>,
    ctxs: Vec<Context<'a>>,
    bus: Option<Channel>,
}

impl<'a, T> Scheduler<'a, T>
    where T: Task<'a>
{
    pub fn new() -> Self {
        Scheduler {
            tasks: Vec::with_capacity(TASKS_MAX_CNT),
            ctxs: Vec::with_capacity(TASKS_MAX_CNT),
            bus: None,
        }
    }

    pub fn with_channel(c: Channel) -> Self {
        Scheduler {
            tasks: Vec::with_capacity(TASKS_MAX_CNT),
            ctxs: Vec::with_capacity(TASKS_MAX_CNT),
            bus: Some(c),
        }
    }

    pub fn spawn(&'a mut self, t: T, l: TaskTermination, input: Option<&'a str>) -> TaskId {
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
    fn poll_bus(&mut self) {
        if let Some(ref bus) = self.bus {
            for s in &bus.subscribers {
                // if bus.id == 1 {
                //     return;
                // }
                match s.recv() {
                    Some(v) => {
                        println!("poll bus on core_{} {:?}", bus.id, v);
                        s.commit();
                    }
                    None => {}
                }
            }
        }
    }

    pub fn run(&mut self) -> Poll<Context<'a>, task::Error> {
        let f: *mut Self = self;
        loop {
            let h1: &mut Self = unsafe { &mut *f };
            h1.poll_bus();
            for (i, t) in h1.tasks.iter_mut().enumerate() {
                let c = h1.ctxs.get_mut(i).expect("Scheduler: can't retrieve a ctx.");
                let mut ctx = mem::replace(c, Context::Nil);
                match t.0.poll(ctx) {
                    Poll::Yield(..) => (),
                    Poll::End(v) => {
                        let h2: &mut Self = unsafe { &mut *f };
                        h2.terminate(t.1, i);
                        return Poll::End(v);
                    }
                    Poll::Err(e) => {
                        let h2: &mut Self = unsafe { &mut *f };
                        h2.terminate(t.1, i);
                        return Poll::Err(e);
                    }
                }
            }
        }
    }
}

impl<'a, T> PubSub<Message> for Scheduler<'a, T> {
    fn subscribe(&mut self) -> Subscriber<Message> {
        self.bus.as_mut().expect("This scheduler without bus!").publisher.subscribe()
    }

    fn add_subscriber(&mut self, s: Subscriber<Message>) {
        self.bus.as_mut().expect("This scheduler without bus!").subscribers.push(s);
    }
}
