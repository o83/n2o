use streams::sched::task::{Task, Context};
use std::mem;
use ptr::*;

const TASKS_MAX_CNT: usize = 256;

pub struct Reactor<'a, T: 'a> {
    tasks: Vec<T>,
    ctxs: Vec<Context<'a>>,
}

impl<'a, T> Reactor<'a, T>
    where T: Task<'a>
{
    pub fn new() -> Self {
        Reactor {
            tasks: Vec::with_capacity(TASKS_MAX_CNT),
            ctxs: Vec::with_capacity(TASKS_MAX_CNT),
        }
    }

    pub fn spawn(&'a mut self, t: T, input: Option<&'a str>) {
        self.tasks.push(t);
        self.ctxs.push(Context::Nil);
        self.tasks.last_mut().unwrap().init(input);
    }

    pub fn run(&'a mut self) {
        let f: *mut Self = self;
        loop {
            let h1: &mut Self = unsafe { &mut *f };
            for (i, t) in h1.tasks.iter_mut().enumerate() {
                let mut ctx = mem::replace(h1.ctxs.get_mut(i).unwrap(), Context::Nil);
                let r = t.poll(ctx);
                println!("Task Poll: {:?}", r);
            }
        }
    }
}
