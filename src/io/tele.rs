
use std::io::{self, ErrorKind, Error, Read};
use std::rc::Rc;
use io::token::Token;
use io::ready::Ready;
use io::poll::*;
use io::options::*;
use slab;
use core::mem::transmute;
use core::ptr::copy_nonoverlapping;
use super::stdio;

pub struct Tele {
    stdin: stdio::Stdin,
    pub token: Token,
    interest: Ready,
    send_queue: Vec<Rc<Vec<u8>>>,
    read_continuation: Option<u64>,
    write_continuation: bool,
}

impl Tele {
    pub fn new(token: Token) -> Self {
        Tele {
            stdin: stdio::Stdin::new(),
            token: token,
            interest: Ready::hup(),
            send_queue: Vec::new(),
            read_continuation: None,
            write_continuation: false,
        }
    }

    pub fn register(&mut self, poll: &mut Poll) -> io::Result<()> {
        println!("connection register; token={:?}", self.token);

        self.interest.insert(Ready::readable());

        poll.register(
            &self.stdin,
            self.token,
            self.interest,
            PollOpt::edge() | PollOpt::oneshot()
            ).and_then(|(),| {
            Ok(())
        }).or_else(|e| {
                println!("Failed to reregister {:?}, {:?}", self.token, e);
                Err(e)
            })
    }

    /// Re-register interest in read events with poll.
    pub fn reregister(&mut self, poll: &mut Poll) -> io::Result<()> {
        println!("connection reregister; token={:?}", self.token);

        poll.reregister(
            &self.stdin,
            self.token,
            self.interest,
            PollOpt::edge() | PollOpt::oneshot()
            ).and_then(|(),| {
            Ok(())
        }).or_else(|e| {
                println!("Failed to reregister {:?}, {:?}", self.token, e);
                Err(e)
            })
    }
}
