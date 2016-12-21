use std::net::SocketAddr;
use io::tcp::{TcpListener, TcpStream};
use io::poll::{Poll, Events};
use io::token::Token;
use io::ready::Ready;
use io::options::PollOpt;
use std::io::{Read, Write};
use http_muncher::{Parser, ParserHandler};
use std::collections::HashMap;
use std::str;
use std::fmt;
use rustc_serialize::base64::{ToBase64, STANDARD};
use sha1;

#[derive(Debug)]
pub enum Error {
    RuntimeError,
}

struct HttpHandler {
    current_key: Option<String>,
    headers: HashMap<String, String>,
}

impl ParserHandler for HttpHandler {
    fn on_header_field(&mut self, parser: &mut Parser, s: &[u8]) -> bool {
        match str::from_utf8(s) {
            Ok(s) => self.current_key = Some(s.to_string()),
            Err(_) => {}
        }
        true
    }

    fn on_header_value(&mut self, parser: &mut Parser, s: &[u8]) -> bool {
        if self.current_key.is_some() {
            match str::from_utf8(s) {
                Ok(s) => {
                    let key = self.current_key.clone().unwrap();
                    self.headers.insert(key, s.to_string());
                }
                Err(_) => {}
            }
        }
        self.current_key = None;
        true
    }
}

struct HttpParser {
    parser: Parser,
    handler: HttpHandler,
}

impl HttpParser {
    pub fn new() -> Self {
        HttpParser {
            parser: Parser::request(),
            handler: HttpHandler {
                current_key: None,
                headers: HashMap::new(),
            },
        }
    }

    pub fn parse(&mut self, data: &[u8]) -> usize {
        self.parser.parse(&mut self.handler, data)
    }

    pub fn get<'a>(&self, k: &'a str) -> Option<&String> {
        self.handler.headers.get(&String::from(k))
    }
}

pub struct WsClient {
    sock: TcpStream,
    addr: SocketAddr,
    key: Option<String>,
}

pub struct WsServer {
    tokens: usize,
    poll: Poll,
    events: Events,
    tcp: TcpListener,
    clients: Vec<WsClient>,
    parser: HttpParser,
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
            parser: HttpParser::new(),
        }
    }

    #[inline]
    pub fn split<'a>(&'a mut self) -> (&'a mut Self, &'a mut Self) {
        let f: *mut WsServer = self;
        let uf: &mut WsServer = unsafe { &mut *f };
        let us: &mut WsServer = unsafe { &mut *f };
        (uf, us)
    }

    #[inline]
    fn gen_key(key: &String) -> String {
        let mut m = sha1::Sha1::new();
        m.update(key.as_bytes());
        m.update("258EAFA5-E914-47DA-95CA-C5AB0DC85B11".as_bytes());

        let b = m.digest().bytes();
        return b.to_base64(STANDARD);
    }

    #[inline]
    fn handshake_response(&mut self) {
        // let headers = self.headers.borrow();
        // let response_key = gen_key(&headers.get("Sec-WebSocket-Key").unwrap());
        // let response = fmt::format(format_args!("HTTP/1.1 101 Switching Protocols\r\nConnection: \
        //                                          Upgrade\r\nSec-WebSocket-Accept: {}\r\nUpgrade: websocket\r\n\r\n",
        //                                         response_key));
        // self.socket.try_write(response.as_bytes()).unwrap();
        // self.state.update_open();
    }

    #[inline]
    fn handshake(&mut self, c: &mut WsClient) {
        let mut buf = [0u8; 2048];
        c.sock.read(&mut buf);
        self.parser.parse(&buf);
        let key = Self::gen_key(self.parser.get("Sec-WebSocket-Key").unwrap());
        let response = fmt::format(format_args!("HTTP/1.1 101 Switching Protocols\r\nConnection: \
                                                 Upgrade\r\nSec-WebSocket-Accept: {}\r\nUpgrade: websocket\r\n\r\n",
                                                key));
        c.sock.write(response.as_bytes());
    }

    #[inline]
    fn reg_incoming(&mut self) {
        match self.tcp.accept() {
            Ok((mut s, a)) => {
                self.tokens += 1;
                self.poll.register(&s, Token(self.tokens), Ready::readable(), PollOpt::edge());
                self.clients.push(WsClient {
                    sock: s,
                    addr: a,
                    key: None,
                });
            }
            Err(e) => println!("WsError: {:?}", e),
        }
    }

    #[inline]
    fn read_incoming(&mut self, id: usize) {
        let (s1, s2) = self.split();
        let mut c = s1.clients.get_mut(id - 1).unwrap();
        s2.handshake(c)
    }

    pub fn listen(&mut self) -> Result<(), Error> {
        println!("Listening on {:?}...", self.tcp.local_addr().unwrap());
        loop {
            self.poll.poll(&mut self.events, None).unwrap();
            let (s1, s2) = self.split();
            for event in s1.events.iter() {
                match event.token() {
                    Token(0) => s2.reg_incoming(),
                    Token(id) => s2.read_incoming(id),
                }
            }
        }
        Ok(())
    }
}