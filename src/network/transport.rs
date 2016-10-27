
use std::io::Result;
use mio::Ready;
use network::device::RequestSender;
use network::message::Message;
use reactors::dispatcher;
use network::endpoint::{self, Context, Endpoint, EndpointId, EndpointDesc, EndpointSpec};
use std::rc::Rc;

pub struct Destination<'a> {
    pub addr: &'a str,
    pub pids: (u16, u16),
    pub tcp_no_delay: bool,
    pub recv_max_size: u64,
}
// pub struct Endpoint {
// request_sender: RequestSender,
// remote: bool,
// }
//
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


impl endpoint::Pipe {
    pub fn new_connected(id: EndpointId, url: String, desc: EndpointDesc) -> endpoint::Pipe {
        endpoint::Pipe(Endpoint::new_created(id, url, desc))
    }

    pub fn new_accepted(id: EndpointId, desc: EndpointDesc) -> endpoint::Pipe {
        endpoint::Pipe(Endpoint::new_accepted(id, desc))
    }

    pub fn from_spec(id: EndpointId, spec: EndpointSpec) -> endpoint::Pipe {
        endpoint::Pipe(Endpoint::from_spec(id, spec))
    }

    pub fn open(&self, network: &mut dispatcher::Context) {
        self.0.open(network, true)
    }
    pub fn send(&self, network: &mut dispatcher::Context, msg: Rc<Message>) {
        self.0.send(network, msg)
    }
    pub fn recv(&self, network: &mut dispatcher::Context) {
        self.0.recv(network)
    }
    pub fn close(self, network: &mut dispatcher::Context) -> Option<EndpointSpec> {
        self.0.close(network, true)
    }
    pub fn get_send_priority(&self) -> u8 {
        self.0.get_send_priority()
    }
    pub fn get_recv_priority(&self) -> u8 {
        self.0.get_recv_priority()
    }
}

impl endpoint::Acceptor {
    pub fn new(id: EndpointId, url: String, desc: EndpointDesc) -> endpoint::Acceptor {
        endpoint::Acceptor(Endpoint::new_created(id, url, desc))
    }
    pub fn from_spec(id: EndpointId, spec: EndpointSpec) -> endpoint::Acceptor {
        endpoint::Acceptor(Endpoint::from_spec(id, spec))
    }
    pub fn open(&self, network: &mut dispatcher::Context) {
        self.0.open(network, false)
    }
    pub fn close(self, network: &mut dispatcher::Context) -> Option<EndpointSpec> {
        self.0.close(network, false)
    }
    pub fn get_send_priority(&self) -> u8 {
        self.0.get_send_priority()
    }
    pub fn get_recv_priority(&self) -> u8 {
        self.0.get_recv_priority()
    }
}
