use std::rc::Rc;
use streams::intercore::ctx::{Ctx, Ctxs};
use queues::publisher::Publisher;
use queues::publisher::Subscriber;
use reactors::core::{Async, Core};
use reactors::selector::{Selector, Slot};
use reactors::console::Console;
use reactors::ws::WsServer;
use reactors::task::Task;
use reactors::cpstask::CpsTask;
use reactors::scheduler::{self, Scheduler, TaskTermination};
use std::io::Read;
use handle;
use std::str;

pub struct Hub<'a> {
    core: Core,
    scheduler: Scheduler<'a, CpsTask<'a>>,
    ctx: Rc<Ctx<u64>>,
    cps: CpsTask<'a>,
}

impl<'a> Hub<'a> {
    pub fn new(ctx: Rc<Ctx<u64>>) -> Self {
        Hub {
            core: Core::new(),
            scheduler: Scheduler::new(),
            ctx: ctx.clone(),
            cps: CpsTask::new(ctx.clone()),
        }
    }

    pub fn add_selected(&'a mut self, s: Selector) {
        self.core.spawn(s);
    }

    pub fn boil(&'a mut self) {
        let cps = CpsTask::new(self.ctx.clone());
        let h: *mut Hub<'a> = self;
        let h0: &mut Hub<'a> = unsafe { &mut *h };
        let task_id = h0.scheduler.spawn(cps, TaskTermination::Corecursive, None);
        loop {
            let h1: &mut Hub<'a> = unsafe { &mut *h };
            match h1.core.poll() {
                Async::Ready((i, s)) => {
                    let h2: &mut Hub<'a> = unsafe { &mut *h };
                    let h3: &mut Hub<'a> = unsafe { &mut *h };
                    let h4: &mut Hub<'a> = unsafe { &mut *h };
                    if s.len() == 1 && s[0] == 0x0A {
                        // FIXME: Slot can be non-0
                        h2.core.write(Slot(0), &[0u8; 0]);
                    } else {
                        let x = str::from_utf8(s).unwrap();
                        h3.scheduler.exec(task_id, Some(x));
                        let r = h4.scheduler.run();
                        self.core.write(Slot(0), format!("{:?}\n", r).as_bytes());
                    }
                }
                x => println!("{:?}", x),
            }
        }
    }
}