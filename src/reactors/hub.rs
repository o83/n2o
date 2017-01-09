use std::rc::Rc;
use streams::intercore::ctx::{Ctx, Ctxs};
use queues::publisher::Publisher;
use queues::publisher::Subscriber;
use reactors::core::{Async, Core};
use reactors::selector::Selector;
use reactors::console::Console;
use reactors::ws::WsServer;
use reactors::cpstask::CpsTask;
use reactors::scheduler::{self, Scheduler};
use std::io::Read;
use handle;
use std::str;

pub struct Hub<'a> {
    core: Core,
    scheduler: Scheduler<'a, CpsTask<'a>>,
    ctx: Rc<Ctx<u64>>,
}

impl<'a> Hub<'a> {
    pub fn new(ctx: Rc<Ctx<u64>>) -> Self {
        Hub {
            core: Core::new(),
            scheduler: Scheduler::new(),
            ctx: ctx,
        }
    }

    pub fn add_selected(&'a mut self, s: Selector) {
        self.core.spawn(s);
    }

    pub fn exec(&'a mut self, input: Option<&'a str>) {
        self.scheduler.spawn(CpsTask::new(self.ctx.clone()), input);
    }

    pub fn boil(&'a mut self) {
        let h: *mut Hub<'a> = self;
        loop {
            let h1: &mut Hub<'a> = unsafe { &mut *h };
            match h1.core.poll() {
                Async::Ready((i, s)) => {
                    let h2: &mut Hub<'a> = unsafe { &mut *h };
                    let h3: &mut Hub<'a> = unsafe { &mut *h };
                    h2.exec(Some(str::from_utf8(s).unwrap()));
                    h3.scheduler.run();
                    // h.borrow_mut().write(i, b"170");
                }
                x => println!("{:?}", x),
            }
        }
    }
}