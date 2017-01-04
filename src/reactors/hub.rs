use std::net::SocketAddr;
use io::ws::*;
use io::console::Console;
use io::reception::{self, Select, Selected, Handle};
use streams::sched::iprtask::IprTask;
use streams::sched::reactor::{self, Reactor};
use std::rc::Rc;
use streams::intercore::ctx::{Ctx, Ctxs};
use queues::publisher::Publisher;
use queues::publisher::Subscriber;

pub struct Hub<'a> {
    reactor: Reactor<'a, IprTask<'a>>,
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
            reactor: Reactor::new(),
            ctx: Rc::new(ctx),
        }
    }

    pub fn exec(&'a mut self, input: Option<&'a str>) {
        self.reactor.spawn(IprTask::new(self.ctx.clone()), input);
    }

    pub fn boil(&'a mut self) {
        self.reactor.run();
    }
}