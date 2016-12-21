use std::net::SocketAddr;
use io::tcp::{TcpListener, TcpStream};
use io::poll::{Poll, Events};
use io::token::Token;
use io::ready::Ready;
use io::options::PollOpt;
use std::io::Read;

#[derive(Debug)]
pub enum Error {
    RuntimeError,
}

pub struct WsClient {
    sock: TcpStream,
    addr: SocketAddr,
}

pub struct WsServer {
    tokens: usize,
    poll: Poll,
    events: Events,
    tcp: TcpListener,
    clients: Vec<WsClient>,
}

impl WsServer {
    pub fn new(addr: &SocketAddr) -> Self {
        let p = Poll::new().unwrap();
        let t = TcpListener::bind(&addr).unwrap();
        p.register(&t, Token(0), Ready::readable(), PollOpt::edge()).unwrap();
        WsServer {
            tokens: 0,
            poll: p,
            events: Events::with_capacity(1024),
            tcp: t,
            clients: Vec::with_capacity(256),
        }
    }

    #[inline]
    pub fn split<'a>(&'a mut self) -> (&'a mut Self, &'a mut Self) {
        let f: *mut WsServer = self;
        let uf: &mut WsServer = unsafe { &mut *f };
        let us: &mut WsServer = unsafe { &mut *f };
        (uf, us)
    }

    fn handshake(c: &mut WsClient) {
        let mut buf = [0u8; 2048];
        c.sock.read(&mut buf);
        println!("Handshake");
        for b in buf.iter() {
            println!("{}", b);
        }
    }

    #[inline]
    fn reg_incoming(&mut self) {
        match self.tcp.accept() {
            Ok((mut s, a)) => {
                self.tokens += 1;
                self.poll.register(&s, Token(self.tokens), Ready::readable(), PollOpt::edge());
                self.clients.push(WsClient { sock: s, addr: a });
            }
            Err(e) => println!("WsError: {:?}", e),
        }
    }

    #[inline]
    fn read_incoming(&mut self) {
        let mut c = self.clients.last_mut().unwrap();
        Self::handshake(c)
    }

    pub fn listen(&mut self) -> Result<(), Error> {
        println!("Listening on {:?}...\n>", self.tcp.local_addr().unwrap());
        loop {
            self.poll.poll(&mut self.events, None).unwrap();
            let (s1, s2) = self.split();
            for event in s1.events.iter() {
                match event.token() {
                    Token(0) => s2.reg_incoming(),
                    _ => s2.read_incoming(),
                }
            }
        }
        Ok(())
    }
}