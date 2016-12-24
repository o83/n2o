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
use io::event::Evented;
use io::reception::{React, Reception};
use std::marker::PhantomData;

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

pub struct WsServer<'a> {
    reception: &'a mut Reception,
    tcp: TcpListener,
    token: Token,
    clients: HashMap<Token, WsClient>,
    parser: HttpParser,
    pub buf: [u8; BUF_SIZE],
}

impl<'a> WsServer<'a> {
    pub fn new(r: &'a mut Reception, addr: &SocketAddr) -> Self {
        let t = TcpListener::bind(&addr).unwrap();
        let tok = r.register(&t,
                      PollOpt::edge(),
                      Box::new(|t| println!("Token: {:?}", t)))
            .unwrap();
        WsServer {
            reception: r,
            tcp: t,
            token: tok,
            clients: HashMap::new(),
            parser: HttpParser::new(),
            buf: [0u8; BUF_SIZE],
        }
    }

    #[inline]
    pub fn split(&mut self) -> (&mut Self, &mut Self) {
        let f: *mut WsServer<'a> = self;
        let uf: &mut WsServer<'a> = unsafe { &mut *f };
        let us: &mut WsServer<'a> = unsafe { &mut *f };
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
    fn reg_incoming(&mut self) {
        match self.tcp.accept() {
            Ok((mut s, a)) => {
                let t = self.reception
                    .register(&s, PollOpt::edge(), Box::new(|t| println!("T:: {:?}", t)))
                    .unwrap();
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
        let (s1, s2) = self.split();
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

    pub fn write_message(&mut self, payload: &[u8]) {
        let sz = payload.len();
        let mut buf = Vec::<u8>::with_capacity(sz + 2);
        buf.push(130);
        buf.push(sz as u8);
        buf.extend_from_slice(payload);
        for (t, c) in &mut self.clients {
            c.sock.write(&buf);
        }
    }

    pub fn listen<K>(&mut self, mut f: K)
        where K: FnMut((&mut WsServer, &[u8]))
    {
        println!("Listening on {:?}...", self.tcp.local_addr().unwrap());
        // loop {
        //     self.poll.poll(&mut self.events, None).unwrap();
        //     let (s1, s2) = self.split();
        //     for event in s1.events.iter() {
        //         match event.token() {
        //             Token(0) => s2.reg_incoming(),
        //             Token(id) => {
        //                 let sz = s2.read_incoming(id);
        //                 if sz > 0 {
        //                     let (mut s3, s4) = s2.split();
        //                     f((&mut s3, &s4.buf[..sz]));
        //                 }
        //             }
        //         }
        //     }
        // }
    }
}