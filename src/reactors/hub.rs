use std::rc::Rc;
use streams::intercore::ctx::Ctx;
use queues::publisher::Publisher;
use queues::publisher::Subscriber;
use reactors::core::Core;
use reactors::selector::{Selector, Slot, Async, Pool};
use reactors::console::Console;
use reactors::ws::WsServer;
use reactors::task::Task;
use reactors::cpstask::CpsTask;
use reactors::scheduler::{self, Scheduler, TaskTermination, TaskId};
use std::io::Read;
use handle;
use std::str;
use streams::intercore::api::*;

pub struct Hub<'a> {
    core: Core,
    scheduler: Scheduler<'a, CpsTask<'a>>,
    ctx: Rc<Ctx>,
}

impl<'a> Hub<'a> {
    pub fn new(ctx: Rc<Ctx>) -> Self {
        Hub {
            core: Core::new(),
            scheduler: Scheduler::new(),
            ctx: ctx,
        }
    }

    pub fn add_selected(&mut self, s: Selector) {
        self.core.spawn(s);
    }

    #[inline]
    fn handle_raw(&'a mut self, t: TaskId, b: &'a [u8]) {
        if b.len() == 0 {
            return;
        }
        if b.len() == 1 && b[0] == 0x0A {
            self.core.write_all(&[0u8; 0]);
            return;
        }
        let x = str::from_utf8(b).unwrap();
        let (s1, s2) = handle::split(self);
        s1.scheduler.exec(t, Some(x));
        let r = s2.scheduler.run();
        s2.core.write_all(format!("{:?}\n", r).as_bytes());
    }

    #[inline]
    fn ready(&'a mut self, p: Pool<'a>, t: TaskId) {
        match p {
            Pool::Raw(b) => self.handle_raw(t, b),            
            Pool::Msg(x) => println!("Intercore: {:?}", x.buf),
        }
    }

    pub fn boil(&mut self) {
        let cps = CpsTask::new(self.ctx.clone());
        let h: *mut Hub<'a> = self;
        let h0: &mut Hub<'a> = unsafe { &mut *h };
        let task_id = h0.scheduler.spawn(cps, TaskTermination::Corecursive, None);
        loop {
            let h1: &mut Hub<'a> = unsafe { &mut *h };
            let h2: &mut Hub<'a> = unsafe { &mut *h };
            match h1.core.poll() {
                Async::Ready((_, p)) => h2.ready(p, task_id),
                Async::NotReady => (),
            }
        }
    }
}