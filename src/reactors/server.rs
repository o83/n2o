
//  Network I/O Reactor by 5HT

use std::io::{self, ErrorKind};
use std::rc::Rc;
use slab;
use io::token::Token;
use io::ready::Ready;
use io::poll::*;
use io::options::*;
use io::tcp::*;
use io::connection::Connection;

type Slab<T> = slab::Slab<T, Token>;

pub struct Server {
    sock: TcpListener,
    token: Token,
    conns: Slab<Connection>,
    events: Events,
}

impl Server {
    pub fn new(sock: TcpListener) -> Server {
        Server {
            sock: sock,
            token: Token(101),
            conns: Slab::with_capacity(128),
            events: Events::with_capacity(1024),
        }
    }

    pub fn run(&mut self, poll: &mut Poll) -> io::Result<()> {

        try!(self.register(poll));

        debug!("Server run loop starting...");
        loop {
            let cnt = try!(poll.poll(&mut self.events, None));

            let mut i = 0;

            println!("processing events... cnt={}; len={}",
                     cnt,
                     self.events.len());

            while i < cnt {
                let event = self.events.get(i).expect("Failed to get event");
                println!("event={:?}; idx={:?}", event, i);
                self.ready(poll, event.token(), event.kind());
                i += 1;
            }

            self.tick(poll);
        }
    }

    pub fn register(&mut self, poll: &mut Poll) -> io::Result<()> {
        poll.register(&self.sock, self.token, Ready::readable(), PollOpt::edge())
            .or_else(|e| {
                println!("Failed to register server {:?}, {:?}", self.token, e);
                Err(e)
            })
    }

    fn tick(&mut self, poll: &mut Poll) {
        println!("Handling end of tick");

        let mut reset_tokens = Vec::new();

        for c in self.conns.iter_mut() {
            if c.is_reset() {
                reset_tokens.push(c.token);
            } else if c.is_idle() {
                c.reregister(poll)
                    .unwrap_or_else(|e| {
                        println!("Reregister failed {:?}", e);
                        c.mark_reset();
                        reset_tokens.push(c.token);
                    });
            }
        }

        for token in reset_tokens {
            match self.conns.remove(token) {
                Some(_c) => {
                    println!("reset connection; token={:?}", token);
                }
                None => {
                    println!("Unable to remove connection for {:?}", token);
                }
            }
        }
    }

    fn ready(&mut self, poll: &mut Poll, token: Token, event: Ready) {
        println!("{:?} event = {:?}", token, event);

        if event.is_error() {
            println!("Error event for {:?}", token);
            self.find_connection_by_token(token).mark_reset();
            return;
        }

        if event.is_hup() {
            println!("Hup event for {:?}", token);
            self.find_connection_by_token(token).mark_reset();
            return;
        }

        if event.is_writable() {
            println!("Write event for {:?}", token);
            assert!(self.token != token, "Received writable event for Server");

            let conn = self.find_connection_by_token(token);

            if conn.is_reset() {
                println!("{:?} has already been reset", token);
                return;
            }

            conn.writable()
                .unwrap_or_else(|e| {
                    println!("Write event failed for {:?}, {:?}", token, e);
                    conn.mark_reset();
                });
        }

        if event.is_readable() {
            println!("Read event for {:?}", token);
            if self.token == token {
                self.accept(poll);
            } else {

                if self.find_connection_by_token(token).is_reset() {
                    println!("{:?} has already been reset", token);
                    return;
                }

                self.readable(token)
                    .unwrap_or_else(|e| {
                        println!("Read event failed for {:?}: {:?}", token, e);
                        self.find_connection_by_token(token).mark_reset();
                    });
            }
        }

        if self.token != token {
            self.find_connection_by_token(token).mark_idle();
        }
    }

    fn accept(&mut self, poll: &mut Poll) {
        println!("server accepting new socket");

        loop {

            let sock = match self.sock.accept() {
                Ok((sock, _)) => sock,
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        println!("accept encountered WouldBlock");
                    } else {
                        println!("Failed to accept new socket, {:?}", e);
                    }
                    return;
                }
            };

            let token = match self.conns.vacant_entry() {
                Some(entry) => {
                    println!("registering {:?} with poller", entry.index());
                    let c = Connection::new(sock, entry.index());
                    entry.insert(c).index()
                }
                None => {
                    println!("Failed to insert connection into slab");
                    return;
                }
            };

            match self.find_connection_by_token(token).register(poll) {
                Ok(_) => {}
                Err(e) => {
                    println!("Failed to register {:?} connection with poller, {:?}",
                             token,
                             e);
                    self.conns.remove(token);
                }
            }
        }
    }

    fn readable(&mut self, token: Token) -> io::Result<()> {
        println!("server conn readable; token={:?}", token);

        while let Some(message) = try!(self.find_connection_by_token(token).readable()) {

            let rc_message = Rc::new(message);

            for c in self.conns.iter_mut() {
                c.send_message(rc_message.clone())
                    .unwrap_or_else(|e| {
                        println!("Failed to queue message for {:?}: {:?}", c.token, e);
                        c.mark_reset();
                    });
            }
        }

        Ok(())
    }

    fn find_connection_by_token<'a>(&'a mut self, token: Token) -> &'a mut Connection {
        &mut self.conns[token]
    }
}
