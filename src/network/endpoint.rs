
use mio::{Evented, Ready, PollOpt};
use std::fmt;
use network::tcp::pipe::Event;

pub struct EndpointTmpl {
    pub pids: (u16, u16),
    pub spec: EndpointSpec,
}

pub struct EndpointSpec {
    pub url: String,
    pub desc: EndpointDesc,
}

pub struct EndpointDesc {
    pub send_priority: u8,
    pub recv_priority: u8,
    pub tcp_no_delay: bool,
    pub recv_max_size: u64,
}

pub trait EndpointRegistrar {
    fn register(&mut self, io: &Evented, interest: Ready, opt: PollOpt);
    fn reregister(&mut self, io: &Evented, interest: Ready, opt: PollOpt);
    fn deregister(&mut self, io: &Evented);
}

pub trait Context: EndpointRegistrar + fmt::Debug {
    fn raise(&mut self, evt: Event);
}
