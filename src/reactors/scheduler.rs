use reactors::task::{Task, Context, Poll};
use std::mem;
use handle::*;

const TASKS_MAX_CNT: usize = 256;

#[derive(Debug,Clone,Copy)]
pub struct TaskId(usize);

#[derive(Debug,PartialEq)]
pub enum TaskLifetime {
    Mortal,
    Immortal,
}

pub struct Scheduler<'a, T: 'a> {
    tasks: Vec<(T, TaskLifetime)>,
    ctxs: Vec<Context<'a>>,
}

impl<'a, T> Scheduler<'a, T>
    where T: Task<'a>
{
    pub fn new() -> Self {
        Scheduler {
            tasks: Vec::with_capacity(TASKS_MAX_CNT),
            ctxs: Vec::with_capacity(TASKS_MAX_CNT),
        }
    }

    pub fn spawn(&'a mut self, t: T, l: TaskLifetime, input: Option<&'a str>) -> TaskId {
        let last = self.tasks.len();
        self.tasks.push((t, l));
        self.ctxs.push(Context::Nil);
        self.tasks.last_mut().unwrap().0.init(input);
        TaskId(last)
    }

    pub fn exec(&'a mut self, t: TaskId, input: Option<&'a str>) {
        self.tasks.get_mut(t.0).unwrap().0.exec(input);
    }

    pub fn run(&'a mut self) {
        let f: *mut Self = self;
        loop {
            let h1: &mut Self = unsafe { &mut *f };
            for (i, t) in h1.tasks.iter_mut().enumerate() {
                let c = h1.ctxs.get_mut(i).unwrap();
                let mut ctx = mem::replace(c, Context::Nil);
                match t.0.poll(ctx) {
                    Poll::Yield(..) => (),
                    Poll::End(v) => {
                        println!("END: {:?}", v);
                        let h2: &mut Self = unsafe { &mut *f };
                        if t.1 == TaskLifetime::Mortal {
                            h2.tasks.remove(i);
                            h2.ctxs.remove(i);
                        }
                        return;
                    }
                    Poll::Err(e) => {
                        println!("{:?}", e);
                        let h2: &mut Self = unsafe { &mut *f };
                        if t.1 == TaskLifetime::Mortal {
                            h2.tasks.remove(i);
                            h2.ctxs.remove(i);
                        }
                        return;
                    }
                }
            }
        }
    }
}
