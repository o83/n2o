use reactors::task::{Task, Context, Poll};
use std::mem;
use handle::*;

const TASKS_MAX_CNT: usize = 256;

pub struct Scheduler<'a, T: 'a> {
    tasks: Vec<T>,
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

    pub fn spawn(&'a mut self, t: T, input: Option<&'a str>) {
        self.tasks.push(t);
        self.ctxs.push(Context::Nil);
        self.tasks.last_mut().unwrap().init(input);
    }

    // pub fn exec(&'a mut self, )

    pub fn run(&'a mut self) {
        let f: *mut Self = self;
        loop {
            let h1: &mut Self = unsafe { &mut *f };
            for (i, t) in h1.tasks.iter_mut().enumerate() {
                let c = h1.ctxs.get_mut(i).unwrap();
                let mut ctx = mem::replace(c, Context::Nil);
                match t.poll(ctx) {
                    Poll::Yield(..) => (),
                    Poll::End(v) => {
                        println!("{:?}", v);
                        let h2: &mut Self = unsafe { &mut *f };
                        h2.tasks.remove(i);
                        h2.ctxs.remove(i);
                        return;
                    }
                    Poll::Err(e) => {
                        println!("{:?}", e);
                        let h2: &mut Self = unsafe { &mut *f };
                        h2.tasks.remove(i);
                        h2.ctxs.remove(i);
                        return;
                    }
                }
            }
        }
    }
}
