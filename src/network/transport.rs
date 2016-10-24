
use std::io::Result;
use mio::Ready;
use network::message::Message;
use network::endpoint::Context;
use std::rc::Rc;

pub struct Destination<'a> {
    pub addr: &'a str,
    pub pids: (u16, u16),
    pub tcp_no_delay: bool,
    pub recv_max_size: u64,
}

pub trait Transport {
    fn connect(&self, dest: &Destination) -> Result<Box<Pipe>>;
    fn bind(&self, dest: &Destination) -> Result<Box<Acceptor>>;
}

pub trait Acceptor {
    fn ready(&mut self, ctx: &mut Context, events: Ready);
    fn open(&mut self, ctx: &mut Context);
    fn close(&mut self, ctx: &mut Context);
}

pub trait Pipe {
    fn ready(&mut self, ctx: &mut Context, events: Ready);
    fn open(&mut self, ctx: &mut Context);
    fn close(&mut self, ctx: &mut Context);
    fn send(&mut self, ctx: &mut Context, msg: Rc<Message>);
    fn recv(&mut self, ctx: &mut Context);
}
