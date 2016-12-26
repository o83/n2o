use std::io;
use io::poll::{Poll, Events};
use io::token::Token;
use io::ready::Ready;
use io::options::PollOpt;
use std::collections::HashMap;
use io::event::Evented;
use io::ws::WsServer;
use std::cell::UnsafeCell;
use reactors::console::Console;
use std::net::SocketAddr;
use io::reception::Selected;

const POLL_SIZE: usize = 1024;
const SUBSCRIBERS_CAPACITY: usize = 256;

pub struct Core<'a> {
    poll: Poll,
    events: Events,
    ws: WsServer<'a>, // console: S,
}

impl<'a> Core<'a> {
    pub fn new(addr: &SocketAddr) -> Self {
        let poll = Poll::new().unwrap();
        let ws = WsServer::new(addr);
        // let console = Console::new();
        Core {
            poll: poll,
            events: Events::with_capacity(1024),
            ws: ws, // console: console,
        }
    }

    pub fn config(&'a mut self) {
        self.ws.initial(&self.poll, 0, POLL_SIZE);
    }

    pub fn poll(&mut self) {

        loop {
            self.poll.poll(&mut self.events, None).unwrap();
            // self.console.select(&self.events);
            self.ws.select(&self.events);
        }
    }
}