
//  Console I/O Reactor by Anton

use std::io::{self, ErrorKind};
use std::io::prelude::*;
use std::rc::Rc;
use io::token::Token;
use io::ready::Ready;
use io::poll::*;
use io::options::*;
use io::tele::*;
use commands::*;

pub struct Console {
    tele: Tele,
    running: bool,
    token: Token,
    events: Events,
}

impl Console {
    pub fn new() -> Self {
        let tok = 102;
        Console {
            tele: Tele::new(Token(tok)),
            token: Token(tok),
            running: true,
            events: Events::with_capacity(1024),
        }
    }

    pub fn prompt() {
        print!("> ");
        let _ = io::stdout().flush();
    }

    pub fn run(&mut self, poll: &mut Poll) -> io::Result<()> {
        try!(self.register(poll));
        println!("Console is listening...");
        loop {
            match self.running {
                false => break,
                _ => (),
            }
            Console::prompt();
            let cnt = try!(poll.poll(&mut self.events, None));
            let mut i = 0;
            trace!("processing events... cnt={}; len={}",
                   cnt,
                   self.events.len());
            while i < cnt {
                let event = self.events.get(i).expect("Failed to get event");
                trace!("event={:?}; idx={:?}", event, i);
                self.ready(poll, event.token(), event.kind());
                i += 1;
            }
        }
        Ok(())
    }

    pub fn register(&mut self, poll: &mut Poll) -> io::Result<()> {
        self.tele
            .register(poll)
            .or_else(|e| {
                println!("Failed to register console {:?}, {:?}", self.token, e);
                Err(e)
            })
    }

    fn ready(&mut self, poll: &mut Poll, token: Token, event: Ready) {
        trace!("{:?} event = {:?}", token, event);

        if event.is_error() {
            error!("Error event for {:?}", token);
            return;
        }

        if event.is_hup() {
            trace!("Hup event for {:?}", token);
            return;
        }

        if event.is_readable() {
            trace!("Read event for {:?}", token);

            self.readable(token)
                .unwrap_or_else(|e| {
                    error!("Read event failed for {:?}: {:?}", token, e);
                });
        }

        self.tele.reregister(poll);
    }

    fn readable(&mut self, token: Token) -> io::Result<()> {
        trace!("console is readable; token={:?}", token);
        let mut msg = [0u8; 128];
        let size = self.tele.read(&mut msg);
        match size {
            Ok(s) => {
                trace!("Read size: {:?}", s);
                let mut m = String::from_utf8_lossy(&msg[..s - 1]);
                match m.trim() {
                    "exit" => self.running = false,
                    line => {
                        println!("{:?}", command::parse_Mex(&line.to_string()));
                    }
                }
            }
            Err(e) => error!("Read error: {:?}.", e),
        }
        Ok(())
    }
}
