
use std::io::{self, ErrorKind, Read};
use std::rc::Rc;
use io::token::Token;
use io::ready::Ready;
use io::poll::*;
use io::options::*;
use io::tele::*;

pub struct Console {
    tele: Tele,
    token: Token,
    events: Events,
}

impl Console {
    pub fn new() -> Self {
        let tok = 10_000_000;
        Console {
            tele: Tele::new(Token(tok)),
            token: Token(tok),
            events: Events::with_capacity(1024),
        }
    }

    pub fn run(&mut self, poll: &mut Poll) -> io::Result<()> {
        try!(self.register(poll));
        println!("Console is listening...");

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
        }
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
        println!("{:?} event = {:?}", token, event);

        if event.is_error() {
            println!("Error event for {:?}", token);
            return;
        }

        if event.is_hup() {
            println!("Hup event for {:?}", token);
            return;
        }

        if event.is_readable() {
            println!("Read event for {:?}", token);

            self.readable(token)
                .unwrap_or_else(|e| {
                    println!("Read event failed for {:?}: {:?}", token, e);
                });
        }

        self.tele.reregister(poll);
    }

    fn readable(&mut self, token: Token) -> io::Result<()> {
        println!("console is readable; token={:?}", token);
        let mut msg = [0u8; 128];
        let size = self.tele.read(&mut msg);
        match size {
            Ok(s) => {
                println!("Read size: {:?}", s);
                let m = String::from_utf8_lossy(&msg[..s - 1]);
                println!("Message: {:?}", m);
            }
            Err(e) => println!("Read error: {:?}.", e),
        }
        Ok(())
    }
}
