use std::net::SocketAddr;
use io::ws::*;
use io::console::Console;
use io::reception::{self, Select, Selected, Handle};
use streams::sched::iprtask::IprTask;
use streams::sched::reactor::{self, Reactor};

pub struct Hub<'a> {
    reactor: Reactor<'a, IprTask<'a>>,
}

impl<'a> Hub<'a> {
    pub fn new() -> Self {
        Hub { reactor: Reactor::new() }
    }

    pub fn exec(&'a mut self, input: Option<&'a str>) {
        self.reactor.spawn(IprTask::new(), input);
    }

    pub fn boil(&'a mut self) {
        self.reactor.run();
    }
}