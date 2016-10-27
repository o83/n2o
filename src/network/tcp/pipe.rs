use std::ops::Deref;
use std::rc::Rc;
use std::fmt;
use mio::{self, Evented};
use core::mem::transmute;
use network::endpoint::EndpointRegistrar;
use core::ptr::copy_nonoverlapping;
use std::io::{self, Result, Error, ErrorKind, Read, Write};
use mio::tcp::{TcpStream, Shutdown};
use network::tcp::send::SendOperation;
use network::tcp::recv::RecvOperation;
use network::message::Message;
use network::tcp::handshake::*;
use reactors::dispatcher;
use network::endpoint;
use network::transport::Pipe;
use network::tcp::state::transition;
use network::tcp::state::PipeState;

pub enum Command {
    Open,
    Close,
    Send(Rc<Message>),
    Recv,
}

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Command {
    fn name(&self) -> &'static str {
        match *self {
            Command::Open => "Open",
            Command::Close => "Close",
            Command::Send(_) => "Send",
            Command::Recv => "Recv",
        }
    }
}


pub enum Event {
    Opened,
    //   Check,
    Closed,
    CanSend,
    CanRecv,
    Sent,
    // Close(bool),
    Received(Message),
    Accepted(Vec<Box<Pipe>>),
    Error(io::Error),
}

pub struct TcpPipeStub {
    stream: TcpStream,
    recv_max_size: u64,
    send_operation: Option<SendOperation>,
    recv_operation: Option<RecvOperation>,
}

impl Deref for TcpPipeStub {
    type Target = mio::Evented;
    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}

impl TcpPipeStub {
    pub fn new(stream: TcpStream, recv_max_size: u64) -> TcpPipeStub {
        TcpPipeStub {
            stream: stream,
            recv_max_size: recv_max_size,
            send_operation: None,
            recv_operation: None,
        }
    }

    fn run_send_operation(&mut self, mut send_operation: SendOperation) -> io::Result<bool> {
        if try!(send_operation.run(&mut self.stream)) {
            Ok(true)
        } else {
            self.send_operation = Some(send_operation);
            Ok(false)
        }
    }

    fn run_recv_operation(&mut self,
                          mut recv_operation: RecvOperation)
                          -> io::Result<Option<Message>> {
        match try!(recv_operation.run(&mut self.stream)) {
            Some(msg) => Ok(Some(msg)),
            None => {
                self.recv_operation = Some(recv_operation);
                Ok(None)
            }
        }
    }
}

impl Drop for TcpPipeStub {
    fn drop(&mut self) {
        let _ = self.stream.shutdown(Shutdown::Both);
    }
}

impl Sender for TcpPipeStub {
    fn start_send(&mut self, msg: Rc<Message>) -> io::Result<bool> {
        let send_operation = SendOperation::new(msg);
        self.run_send_operation(send_operation)
    }

    fn resume_send(&mut self) -> io::Result<bool> {
        if let Some(send_operation) = self.send_operation.take() {
            self.run_send_operation(send_operation)
        } else {
            Err(Error::new(ErrorKind::Other, "Cannot resume send: no pending operation"))
        }
    }

    fn has_pending_send(&self) -> bool {
        self.send_operation.is_some()
    }
}

impl Receiver for TcpPipeStub {
    fn start_recv(&mut self) -> io::Result<Option<Message>> {
        let recv_operation = RecvOperation::new(self.recv_max_size);
        self.run_recv_operation(recv_operation)
    }

    fn resume_recv(&mut self) -> io::Result<Option<Message>> {
        if let Some(recv_operation) = self.recv_operation.take() {
            self.run_recv_operation(recv_operation)
        } else {
            Err(Error::new(ErrorKind::Other, "Cannot resume recv: no pending operation"))
        }
    }

    fn has_pending_recv(&self) -> bool {
        self.recv_operation.is_some()
    }
}

impl Handshake for TcpPipeStub {
    fn send_handshake(&mut self, pids: (u16, u16)) -> io::Result<()> {
        send_and_check_handshake(&mut self.stream, pids)
    }
    fn recv_handshake(&mut self, pids: (u16, u16)) -> io::Result<()> {
        recv_and_check_handshake(&mut self.stream, pids)
    }
}

pub trait Sender {
    fn start_send(&mut self, msg: Rc<Message>) -> Result<bool>;
    fn resume_send(&mut self) -> Result<bool>;
    fn has_pending_send(&self) -> bool;
}

pub trait Receiver {
    fn start_recv(&mut self) -> Result<Option<Message>>;
    fn resume_recv(&mut self) -> Result<Option<Message>>;
    fn has_pending_recv(&self) -> bool;
}

pub trait Handshake {
    fn send_handshake(&mut self, pids: (u16, u16)) -> Result<()>;
    fn recv_handshake(&mut self, pids: (u16, u16)) -> Result<()>;
}

pub fn send_and_check_handshake<T: Write>(stream: &mut T, pids: (u16, u16)) -> Result<()> {
    let (proto_id, _) = pids;
    let handshake = create_handshake(proto_id);

    match try!(stream.write(&handshake)) {
        8 => Ok(()),
        _ => Err(Error::new(ErrorKind::Other, "failed to send handshake")),
    }
}

fn create_handshake(protocol_id: u16) -> [u8; 8] {
    // handshake is Zero, 'S', 'P', Version, Proto[2], Rsvd[2]
    let mut handshake = [0, 83, 80, 0, 0, 0, 0, 0];
    unsafe {
        let bytes = transmute::<_, [u8; 2]>(protocol_id.to_be());
        copy_nonoverlapping((&bytes).as_ptr(), (&mut handshake[4..6]).as_mut_ptr(), 2);
    }
    // write_num_bytes!(u16, 2, n, buf, to_be);
    // BigEndian::write_u16(&mut handshake[4..6], protocol_id);
    handshake
}

pub fn recv_and_check_handshake<T: Read>(stream: &mut T, pids: (u16, u16)) -> Result<()> {
    let mut handshake = [0u8; 8];
    stream.read(&mut handshake).and_then(|_| check_handshake(pids, &handshake))
}

fn check_handshake(pids: (u16, u16), handshake: &[u8; 8]) -> Result<()> {
    let (_, proto_id) = pids;
    let expected_handshake = create_handshake(proto_id);

    if handshake == &expected_handshake {
        Ok(())
    } else {
        Err(Error::new(ErrorKind::Other, "received bad handshake"))
    }
}

pub trait WriteBuffer {
    fn write_buffer(&mut self, buffer: &[u8], written: &mut usize) -> Result<bool>;
}

impl<T: Write> WriteBuffer for T {
    fn write_buffer(&mut self, buf: &[u8], written: &mut usize) -> Result<bool> {
        match self.write(&buf[*written..]) {
            Ok(x) => {
                *written += x;
                Ok(*written == buf.len())
            }
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    Ok(false)
                } else {
                    Err(e)
                }
            }
        }
    }
}


pub trait ReadBuffer {
    fn read_buffer(&mut self, buffer: &mut [u8]) -> Result<usize>;
}

impl<T: Read> ReadBuffer for T {
    fn read_buffer(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self.read(buf) {
            Ok(x) => Ok(x),
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    Ok(0)
                } else {
                    Err(e)
                }
            }
        }
    }
}

pub struct AsyncPipe<S: AsyncPipeStub + 'static> {
    state: Option<Box<PipeState<S>>>,
}

pub struct Initial<S: AsyncPipeStub> {
    stub: S,
    proto_ids: (u16, u16),
}

impl<S: AsyncPipeStub> Initial<S> {
    pub fn new(s: S, pids: (u16, u16)) -> Initial<S> {
        Initial {
            stub: s,
            proto_ids: pids,
        }
    }
}

impl<S: AsyncPipeStub + 'static> PipeState<S> for Initial<S> {
    fn name(&self) -> &'static str {
        "Initial"
    }

    fn open(self: Box<Self>, ctx: &mut endpoint::Context) -> Box<PipeState<S>> {
        transition::<Initial<S>, HandshakeTx<S>, S>(self, ctx)
    }
}

impl<S: AsyncPipeStub> Into<HandshakeTx<S>> for Initial<S> {
    fn into(self) -> HandshakeTx<S> {
        HandshakeTx::new(self.stub, self.proto_ids)
    }
}



pub trait AsyncPipeStub: Sender + Receiver + Handshake + Deref<Target = Evented> {}

impl AsyncPipeStub for TcpPipeStub {}

impl<S: AsyncPipeStub + 'static> AsyncPipe<S> {
    pub fn new(stub: S, pids: (u16, u16)) -> AsyncPipe<S> {
        let initial_state = box Initial::new(stub, pids);
        AsyncPipe { state: Some(initial_state) }
    }

    fn apply<F>(&mut self, ctx: &mut endpoint::Context, transition: F)
        where F: FnOnce(Box<PipeState<S>>, &mut endpoint::Context) -> Box<PipeState<S>>
    {
        if let Some(old_state) = self.state.take() {
            let new_state = transition(old_state, ctx);
            self.state = Some(new_state);
        }
    }
}

impl<S: AsyncPipeStub> Pipe for AsyncPipe<S> {
    fn ready(&mut self, ctx: &mut endpoint::Context, events: mio::Ready) {
        self.apply(ctx, |s, ctx| s.ready(ctx, events))
    }

    fn open(&mut self, ctx: &mut endpoint::Context) {
        self.apply(ctx, |s, ctx| s.open(ctx))
    }

    fn close(&mut self, ctx: &mut endpoint::Context) {
        self.apply(ctx, |s, ctx| s.close(ctx))
    }

    fn send(&mut self, ctx: &mut endpoint::Context, msg: Rc<Message>) {
        self.apply(ctx, |s, ctx| s.send(ctx, msg))
    }

    fn recv(&mut self, ctx: &mut endpoint::Context) {
        self.apply(ctx, |s, ctx| s.recv(ctx))
    }
}
