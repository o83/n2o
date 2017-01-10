use std::rc::Rc;
use reactors::core::{Async, Core};
use reactors::scheduler::Scheduler;
use streams::intercore::ctx::Ctx;
use reactors::hub::Hub;
use std::mem;
use handle::{self, Handle};
use reactors::job::Job;
// TODO: next uses will be removed when Interpreter
// could create IO's dynamically.
use reactors::console::Console;
use reactors::ws::WsServer;
use std::net::SocketAddr;
use reactors::selector::{Select, Selector};

pub struct Host<'a> {
    schedulers: Vec<Scheduler<'a, Job<'a>>>,
    junk: Handle<Hub<'a>>,
    rings: Vec<Rc<Ctx>>,
    cores: Vec<Core>,
}

impl<'a> Host<'a> {
    pub fn new() -> Self {
        let mut ctxs = Vec::new();
        ctxs.push(Rc::new(Ctx::new()));
        Host {
            schedulers: Vec::new(),
            junk: handle::new(Hub::new(ctxs.last().unwrap().clone())),
            rings: ctxs,
            cores: Vec::new(),
        }
    }

    pub fn run(&'a mut self) {
        let mut o = Selector::Rx(Console::new());
        let addr = "0.0.0.0:9001".parse::<SocketAddr>().ok().expect("Parser Error");
        let mut w = Selector::Ws(WsServer::new(&addr));
        self.junk.borrow_mut().add_selected(o);
        self.junk.borrow_mut().add_selected(w);
        self.junk.borrow_mut().boil()
    }
}