
//  Console I/O Reactor by Anton

#![allow(unused_must_use)]

use std::io::{self, ErrorKind};
use std::io::prelude::*;
use std::rc::Rc;
use io::token::Token;
use io::ready::Ready;
use io::poll::*;
use io::options::*;
use io::tele::*;
use commands::*;
use commands::ast::*;
use streams::interpreter::Interpreter;
use std::cell::UnsafeCell;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub struct Console<'ast> {
    tele: Tele,
    running: bool,
    token: Token,
    events: Events,
    interpreter: Interpreter<'ast>,
}

impl<'ast> Console<'ast> {
    pub fn new() -> Self {
        let tok = 10_000_000;
        Console {
            tele: Tele::new(Token(tok)),
            token: Token(tok),
            running: true,
            events: Events::with_capacity(1024),
            interpreter: Interpreter::new().unwrap(),
        }
    }

    pub fn prompt() {
        print!("> ");
        let _ = io::stdout().flush();
    }

    pub fn run(&mut self, poll: &mut Poll) -> io::Result<()> {
        try!(self.register(poll));
        println!("Welcome to O-CPS Interpreter v{}!", VERSION.to_string());
        while self.running {
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
        println!("Bye!");
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

            let res = self.readable(token);
            match res {
                Ok(r) => {
                    if !r {
                        self.running = false;
                        return;
                    }
                }
                Err(e) => error!("Read event failed for {:?}: {:?}", token, e),
            };

            self.tele.reregister(poll);
        }
    }

    fn interpreter_run(&'ast mut self, text: String) {
        let x = self.interpreter.parse(&text);
        match self.interpreter.run(x) {
            Ok(r) => println!("{}", r),
            Err(e) => print!("{}", e),
        }
    }

    fn readable(&mut self, token: Token) -> io::Result<bool> {
        trace!("console is readable; token={:?}", token);
        let mut msg = [0u8; 128];
        let size = self.tele.read(&mut msg);
        match size {
            Ok(s) => {
                trace!("Read size: {:?}", &s);
                match s {
                    0 => Ok(false),
                    _ => {
                        let mut m = String::from_utf8_lossy(&msg[..s]);
                        match m.trim() {
                            "exit" => Ok(false),
                            "" => {
                                println!("{}", AST::Nil);
                                Ok(true)
                            }
                            line => {
                                let i = Interpreter::new().unwrap();
                                let x = i.parse(&line.to_string());
                                match i.run(x) {
                                    Ok(r) => println!("{}", r),
                                    Err(e) => print!("{}", e),
                                }
                                // let _ = self.interpreter_run(line.to_string());
                                Ok(true)
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("Read error: {:?}.", &e);
                Err(e)
            }
        }
    }

    pub fn read_lines<R: BufRead>(&'ast mut self, config: R) -> io::Result<()> {
        for line in config.lines() {
            match line.unwrap().trim() {
                "" => {
                    println!("{}", AST::Nil);
                }
                line => (),
                // line => self.interpreter_run(line.to_string()),
            }
        }
        Ok(())
    }

    pub fn read_all<R: Read>(&mut self, mut config: R) -> io::Result<()> {
        let mut text = String::new();
        try!(config.read_to_string(&mut text));
        // let _ = self.interpreter_run(text.to_string());
        Ok(())
    }
}
