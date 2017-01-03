use streams::sched::task::{Task, Context};
use std::mem;

const TASKS_MAX_CNT: usize = 256;

pub struct Reactor<'a, T> {
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

    pub fn spawn(&'a mut self, t: T) {
        self.tasks.push(t);
        self.ctxs.push(Context::Nil);
        self.tasks.last_mut().unwrap().init();
    }

    pub fn run(&mut self) {
        loop {
            for (i, t) in self.tasks.iter_mut().enumerate() {
                let mut ctx = mem::replace(self.ctxs.get_mut(i).unwrap(), Context::Nil);
                let r = t.poll(ctx);
                print!("Task Poll: {:?}", r);
            }
        }
    }
}