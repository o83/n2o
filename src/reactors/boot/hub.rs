use io::ws::*;
use streams::sched::iprtask::IprTask;
use streams::sched::scheduler::{self, Scheduler};
use std::rc::Rc;
use streams::intercore::ctx::{Ctx, Ctxs};
use queues::publisher::Publisher;
use queues::publisher::Subscriber;
use reactors::boot::reactor::{Async, Core, Selector};
use reactors::boot::console::Console;
use reactors::boot::ws::WsServer;
use std::io::Read;
use handle;

pub struct Hub<'a> {
    core: Core,
    scheduler: Scheduler<'a, IprTask<'a>>,
    ctx: Rc<Ctx<u64>>,
}

impl<'a> Hub<'a> {
    pub fn new() -> Self {
        let ctx = Ctx::new();
        {
            let pubs = ctx.publishers();
            pubs.push(Publisher::with_capacity(8)); //0
            pubs.push(Publisher::with_capacity(8)); //1
            let subs = ctx.subscribers();
            if let Some(p) = pubs.get_mut(0 as usize) {
                subs.push(p.subscribe());
            }
            if let Some(p) = pubs.get_mut(1 as usize) {
                subs.push(p.subscribe());
            }
        }
        Hub {
            core: Core::new(),
            scheduler: Scheduler::new(),
            ctx: Rc::new(ctx),
        }
    }

    pub fn add_selected(&'a mut self, s: Selector) {
        self.core.spawn(s);
    }

    pub fn exec(&'a mut self, input: Option<&'a str>) {
        self.scheduler.spawn(IprTask::new(self.ctx.clone()), input);
    }

    pub fn boil(&'a mut self) {
        loop {
            match self.core.poll() {
                Async::Ready((i, s)) => {
                    println!("Received: {:?}", String::from_utf8_lossy(s));
                    // h.borrow_mut().write(i, b"170");
                }
                x => println!("{:?}", x),
            }
        }
        self.scheduler.run();
    }
}