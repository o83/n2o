use std::net::SocketAddr;
use io::tcp::{TcpListener, TcpStream};
use io::poll::{Poll, Events};
use io::token::Token;
use io::ready::Ready;
use io::options::PollOpt;
use std::io::{self, Read, Write};
use http_muncher::{Parser, ParserHandler};
use std::collections::HashMap;
use std::str;
use std::fmt;
use std::slice;
use std::mem;
use rustc_serialize::base64::{ToBase64, STANDARD};
use sha1;
use reactors::boot::reactor::{Select, Core, Slot};
use std::cell::UnsafeCell;
use std::fmt::Arguments;
use handle::split;

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
            Err(_) => (),
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
                Err(_) => (),
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
    ready: bool,
}

const BUF_SIZE: usize = 2048;

pub struct WsServer {
    listen_token: Token,
    slot: Slot,
    tcp: TcpListener,
    clients: HashMap<Token, WsClient>,
    parser: HttpParser,
    pub buf: [u8; BUF_SIZE],
}

impl WsServer {
    pub fn new(addr: &SocketAddr) -> Self {
        let t = TcpListener::bind(&addr).unwrap();
        WsServer {
            listen_token: Token(0),
            slot: Slot(0),
            tcp: t,
            clients: HashMap::with_capacity(256),
            parser: HttpParser::new(),
            buf: [0u8; BUF_SIZE],
        }
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
    fn handshake(&mut self, c: &mut WsClient) {
        c.sock.read(&mut self.buf);
        self.parser.parse(&self.buf);
        let key = Self::gen_key(self.parser.get("Sec-WebSocket-Key").unwrap());
        let response = fmt::format(format_args!("HTTP/1.1 101 Switching Protocols\r\nConnection: \
                                                 Upgrade\r\nSec-WebSocket-Accept: {}\r\nUpgrade: websocket\r\n\r\n",
                                                key));
        c.sock.write(response.as_bytes());
    }

    #[inline]
    fn reg_incoming<'a>(&mut self, c: &mut Core<'a>) {
        println!("REG INCOMING");
        match self.tcp.accept() {
            Ok((mut s, a)) => {
                let t = c.register(&s, self.slot);
                self.clients.insert(t,
                                    WsClient {
                                        sock: s,
                                        addr: a,
                                        key: None,
                                        ready: false,
                                    });
            }
            Err(e) => println!("WsError: {:?}", e),
        }
    }

    #[inline]
    fn read_incoming(&mut self, t: Token) -> usize {
        println!("READ INCOMING");
        let (s1, s2) = split(self);
        let mut c = s1.clients.get_mut(&t).unwrap();
        if c.ready {
            match c.sock.read(&mut s2.buf) {
                Ok(s) => s,
                Err(_) => 0,
            }
        } else {
            c.ready = true;
            s2.handshake(c);
            0
        }
    }

    pub fn write_to_clients(&mut self, payload: &[u8]) {
        println!("WRITE TO CLIENTS");
        let sz = payload.len();
        let mut buf = Vec::<u8>::with_capacity(sz + 2);
        buf.push(130);
        buf.push(sz as u8);
        buf.extend_from_slice(payload);
        for c in self.clients.iter_mut().filter(|c| c.1.ready) {
            c.1.sock.write(&buf);
        }
    }
}

impl<'a> Select<'a> for WsServer {
    fn init(&mut self, c: &mut Core<'a>, s: Slot) {
        let t = c.register(&self.tcp, s);
        self.listen_token = t;
        self.slot = s;
    }

    fn select(&mut self, c: &mut Core<'a>, t: Token, buf: &mut Vec<u8>) {
        if t == self.listen_token {
            self.reg_incoming(c);
        } else {
            let sz = self.read_incoming(t);
            if sz > 0 {
                println!("READ: {:?}", &self.buf[0]);
            }
        }
    }

    fn finalize(&mut self) {
        println!("Bye!");
    }
}

impl Write for WsServer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_to_clients(buf);
        Ok(1)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        Ok(())
    }
    fn write_fmt(&mut self, fmt: Arguments) -> io::Result<()> {
        Ok(())
    }
}