
use network::endpoint::{self, Reply, SocketId, EndpointId, DeviceId};
use network::tcp::{pipe, acceptor};
use network::session;
use network::socket;
use network::device;
use reactors::dispatcher::Schedulable;

pub enum Signal {
    PipeCmd(SocketId, EndpointId, pipe::Command),
    PipeEvt(SocketId, EndpointId, pipe::Event),
    AcceptorCmd(SocketId, EndpointId, pipe::Command),
    AcceptorEvt(SocketId, EndpointId, pipe::Event),
    SocketEvt(SocketId, pipe::Event),
}

pub enum Request {
    Session(session::Request),
    Socket(SocketId, socket::Request),
    Endpoint(SocketId, EndpointId, endpoint::Request),
    Device(DeviceId, device::Request),
    Shutdown,
}

pub enum Task {
    Socket(SocketId, Schedulable),
}
