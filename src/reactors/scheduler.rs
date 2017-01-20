use reactors::task::{self, Task, Context, Poll};
use reactors::job::Job;
use reactors::cpstask::CpsTask;
use streams::intercore::ctx::Channel;
use queues::publisher::{Publisher, Subscriber};
use queues::pubsub::PubSub;
use streams::intercore::api::*;
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
    pb: Publisher<Message>,
}

impl<'a> Scheduler<'a> {
    pub fn new() -> Self {
        Scheduler {
            tasks: Vec::with_capacity(TASKS_MAX_CNT),
            ctxs: Vec::with_capacity(TASKS_MAX_CNT),
            bus: None,
            pb: Publisher::with_capacity(8),
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
                    Some(&Message::Spawn(ref v)) if v.to == bus.id => {
                        println!("poll bus on core_{} {:?}", bus.id, v);
<<<<<<< Updated upstream
                        handle::with(self, |h| {
                            h.spawn(Job::Cps(CpsTask::new(Rc::new(Ctx::new()))),
                                    TaskTermination::Recursive,
                                    None)
=======
                        with(self, |h| {
                            from_raw(h).spawn(Job::Cps(CpsTask::new(Rc::new(Ctx::new()))),
                                              TaskTermination::Recursive,
                                              Some(&v.txt))
>>>>>>> Stashed changes
                        });
                        s.commit();
                    }
                    Some(&Message::Pub(ref pb)) if pb.to == bus.id => {
                        println!("poll bus on core_{} {:?}", bus.id, pb);
                        s.commit();
                        if let Some(v) = bus.publisher.next() {
                            *v = Message::Ack(Ack {
                                from: bus.id,
                                to: pb.from,
                                task_id: pb.task_id,
                                result_id: 0,
                                subs: self.pb.subscribe(),
                            });
                            if let Some(v) = self.pb.next() {
                                *v = Message::Unknown;
                                println!("send on core_{} {:?}", bus.id, v);
                                self.pb.commit();
                            }
                            bus.publisher.commit();
                        }
                    }

                    Some(&Message::Sub(ref sb)) if sb.to == bus.id => {
                        // println!("poll bus on core_{} {:?}", bus.id, sb);
                        s.commit();
                    } 

                    Some(&Message::Ack(ref a)) => {
                        println!("ACK on core_");
                        if let Some(v) = a.subs.recv() {
                            println!("ACK on core_{} {:?}", bus.id, v);
                            a.subs.commit();
                        }
                        s.commit();
                    } 

                    _ => {}
                }
            }
        }
    }

    pub fn run(&mut self) -> Poll<Context<'a>, task::Error> {
<<<<<<< Updated upstream
        let h = handle::into_raw(self);
=======
        let h = into_raw(self);
        if let Some(ref bus) = with(self, |h| from_raw(h).bus.as_ref()) {
            if let Some(v) = bus.publisher.next() {
                *v = Message::Pub(Pub {
                    from: bus.id,
                    to: bus.id + 1,
                    task_id: 0,
                    name: "pub0".to_string(),
                });

                // v[1] = Message::Sub(Sub {
                //     from: bus.id,
                //     to: bus.id + 1,
                //     task_id: 0,
                //     pub_id: 0,
                // });

                // v[2] = Message::Spawn(Spawn {
                //     from: bus.id,
                //     to: bus.id + 1,
                //     txt: format!("{} + {}", bus.id + 1, bus.id),
                // });

                bus.publisher.commit();
            }
        }
>>>>>>> Stashed changes
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
