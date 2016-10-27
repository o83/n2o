
use mio::{Evented, Ready, PollOpt};
use std::fmt;
use std::sync::mpsc::Sender;
use network::tcp::pipe::Event;
use std::rc::Rc;
use reactors::dispatcher;
use network::message::Message;
use network::device::RequestSender;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct EndpointId(usize);

pub enum Request {
    Close(bool),
}

pub enum Reply {
    Check(bool, bool),
}

impl fmt::Debug for EndpointId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<usize> for EndpointId {
    fn from(value: usize) -> EndpointId {
        EndpointId(value)
    }
}

impl Into<usize> for EndpointId {
    fn into(self) -> usize {
        self.0
    }
}

impl<'x> Into<usize> for &'x EndpointId {
    fn into(self) -> usize {
        self.0
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct DeviceId(usize);

impl fmt::Debug for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<usize> for DeviceId {
    fn from(value: usize) -> DeviceId {
        DeviceId(value)
    }
}

pub struct Device {
    reply_sender: Sender<Reply>,
    left: SocketId,
    right: SocketId,
    left_recv: bool,
    right_recv: bool,
    checking: bool,
}

impl Device {
    pub fn new(reply_tx: Sender<Reply>, l: SocketId, r: SocketId) -> Device {
        Device {
            reply_sender: reply_tx,
            left: l,
            right: r,
            left_recv: false,
            right_recv: false,
            checking: false,
        }
    }

    pub fn check(&mut self) {
        if self.left_recv | self.right_recv {
            self.send_reply();
        } else {
            self.checking = true;
        }
    }

    pub fn on_socket_can_recv(&mut self, sid: SocketId) {
        if sid == self.left {
            self.left_recv = true;
        } else if sid == self.right {
            self.right_recv = true;
        }

        if self.checking {
            self.send_reply();
        }
    }

    fn send_reply(&mut self) {
        let _ = self.reply_sender.send(Reply::Check(self.left_recv, self.right_recv));

        self.left_recv = false;
        self.right_recv = false;
        self.checking = false;
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct SocketId(usize);

impl fmt::Debug for SocketId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<usize> for SocketId {
    fn from(value: usize) -> SocketId {
        SocketId(value)
    }
}

pub struct Endpoint {
    // request_sender: RequestSender,
    // remote: bool,
    id: EndpointId,
    url: Option<String>,
    desc: EndpointDesc,
}

pub struct Pipe(pub Endpoint);
pub struct Acceptor(pub Endpoint);

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

pub trait Context: EndpointRegistrar {
    fn raise(&mut self, evt: Event);
}


impl Endpoint {
    pub fn new_created(id: EndpointId, url: String, desc: EndpointDesc) -> Endpoint {
        Endpoint {
            id: id,
            url: Some(url),
            desc: desc,
        }
    }

    pub fn new_accepted(id: EndpointId, desc: EndpointDesc) -> Endpoint {
        Endpoint {
            id: id,
            url: None,
            desc: desc,
        }
    }

    pub fn from_spec(id: EndpointId, spec: EndpointSpec) -> Endpoint {
        Endpoint {
            id: id,
            url: Some(spec.url),
            desc: spec.desc,
        }
    }

    pub fn open(&self, network: &mut dispatcher::Context, remote: bool) {
        network.open(self.id, remote)
    }
    pub fn send(&self, network: &mut dispatcher::Context, msg: Rc<Message>) {
        network.send(self.id, msg)
    }
    pub fn recv(&self, network: &mut dispatcher::Context) {
        network.recv(self.id)
    }
    pub fn close(mut self,
                 network: &mut dispatcher::Context,
                 remote: bool)
                 -> Option<EndpointSpec> {
        network.close(self.id, remote);

        match self.url.take() {
            Some(url) => {
                Some(EndpointSpec {
                    url: url,
                    desc: self.desc,
                })
            }
            None => None,
        }
    }
    pub fn get_send_priority(&self) -> u8 {
        self.desc.send_priority
    }
    pub fn get_recv_priority(&self) -> u8 {
        self.desc.recv_priority
    }
}
