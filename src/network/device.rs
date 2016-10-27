use std::sync::mpsc::Sender;
use mio;
use std::io::{self, Result, Error, ErrorKind};
use network::endpoint::{SocketId, DeviceId};
use reactors::api;

pub type EventLoopRequestSender = mio::channel::Sender<api::Request>;

pub enum Request {
    Check,
    Close,
}

pub enum Reply {
    Check(bool, bool),
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

pub struct RequestSender {
    req_tx: EventLoopRequestSender,
    device_id: DeviceId,
}

impl RequestSender {
    pub fn new(tx: EventLoopRequestSender, id: DeviceId) -> RequestSender {
        RequestSender {
            req_tx: tx,
            device_id: id,
        }
    }
    fn send(&self, req: Request) -> io::Result<()> {
        self.req_tx
            .send(api::Request::Device(self.device_id, req))
            .map_err(|_| Error::new(ErrorKind::Other, "send error"))
    }
}
