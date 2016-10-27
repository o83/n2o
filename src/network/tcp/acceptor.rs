
use std::io;
use mio;
use mio::tcp::{TcpListener, TcpStream};
use network::endpoint::{self, Context, EndpointRegistrar};
use reactors::dispatcher;
use network::transport::{Pipe, Destination, Acceptor};
use network::tcp::pipe::{Event, AsyncPipe, TcpPipeStub};

pub struct TcpAcceptor {
    listener: TcpListener,
    proto_ids: (u16, u16),
    no_delay: bool,
    recv_max_size: u64,
}

impl TcpAcceptor {
    pub fn new(l: TcpListener, dest: &Destination) -> TcpAcceptor {
        TcpAcceptor {
            listener: l,
            proto_ids: dest.pids,
            no_delay: dest.tcp_no_delay,
            recv_max_size: dest.recv_max_size,
        }
    }

    fn accept(&mut self, ctx: &mut Context) {
        let mut pipes = Vec::new();

        loop {
            match self.listener.accept() {
                Ok((stream, _)) => {
                    let _ = stream.set_nodelay(self.no_delay);
                    let pipe = self.create_pipe(stream);
                    pipes.push(pipe);
                }
                Err(e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        break;
                    } else {
                        ctx.raise(Event::Error(e));
                    }
                }
            }
        }

        if pipes.is_empty() == false {
            ctx.raise(Event::Accepted(pipes));
        }
    }

    fn create_pipe(&self, stream: TcpStream) -> Box<Pipe> {
        let stub = TcpPipeStub::new(stream, self.recv_max_size);
        box AsyncPipe::new(stub, self.proto_ids)
    }
}

impl Acceptor for TcpAcceptor {
    fn ready(&mut self, ctx: &mut endpoint::Context, events: mio::Ready) {
        if events.is_readable() {
            self.accept(ctx);
        }
    }

    fn open(&mut self, ctx: &mut endpoint::Context) {
        ctx.register(&self.listener, mio::Ready::readable(), mio::PollOpt::edge());
        ctx.raise(Event::Opened);
    }

    fn close(&mut self, ctx: &mut endpoint::Context) {
        ctx.deregister(&self.listener);
        ctx.raise(Event::Closed);
    }
}
