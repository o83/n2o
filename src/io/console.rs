
//  Console I/O by Anton

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
use io::reception::Selected;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub struct Console<'a> {
    tele: Option<Tele>,
    running: bool,
    token: Token,
    events: Events,
    poll: Option<&'a Poll>,
    interpreter: UnsafeCell<Interpreter<'a>>,
}

impl<'a> Console<'a> {
    pub fn new() -> Self {
        Console {
            tele: None,
            token: Token(0),
            running: true,
            events: Events::with_capacity(1024),
            poll: None,
            interpreter: UnsafeCell::new(Interpreter::new().unwrap()),
        }
    }

    pub fn prompt() {
        print!("> ");
        let _ = io::stdout().flush();
    }

    pub fn register(&mut self, poll: &Poll) -> io::Result<()> {
        let t = self.token;
        let mut tele = self.tele.as_mut().unwrap();
        tele.register(poll)
            .or_else(|e| {
                println!("Failed to register console {:?}, {:?}", t, e);
                Err(e)
            })
    }

    fn ready(&mut self, poll: &Poll, token: Token, event: Ready) {
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
            let mut tele = self.tele.as_mut().unwrap();
            tele.reregister(poll);
        }
    }

    fn interpreter_run(&mut self, text: String) {
        let i1: &mut Interpreter = unsafe { &mut *self.interpreter.get() };
        let i2: &mut Interpreter = unsafe { &mut *self.interpreter.get() };
        let x = i1.parse(&text);
        match i2.run(x) {
            Ok(r) => println!("{}", r),
            Err(e) => print!("{}", e),
        }
    }

    fn readable(&mut self, token: Token) -> io::Result<bool> {
        trace!("console is readable; token={:?}", token);
        let mut msg = [0u8; 128];
        let s = UnsafeCell::new(self);
        let s1 = unsafe { &mut *s.get() };
        let s2 = unsafe { &mut *s.get() };
        let mut tele = s1.tele.as_mut().unwrap();
        let size = tele.read(&mut msg);
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
                                let _ = s2.interpreter_run(line.to_string());
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

    pub fn read_lines<R: BufRead>(&'a mut self, config: R) -> io::Result<()> {
        for line in config.lines() {
            match line.unwrap().trim() {
                "" => {
                    println!("{}", AST::Nil);
                }
                line => self.interpreter_run(line.to_string()),
            }
        }
        Ok(())
    }

    pub fn read_all<R: Read>(&mut self, mut config: R) -> io::Result<()> {
        let mut text = String::new();
        try!(config.read_to_string(&mut text));
        let _ = self.interpreter_run(text.to_string());
        Ok(())
    }
}

impl<'a> Selected<'a> for Console<'a> {
    fn select(&mut self, events: &Events) -> usize {
        let mut i = 0;
        let cnt = events.len();
        trace!("processing events... cnt={}; len={}", cnt, events.len());
        for event in events.iter() {
            if event.token().0 == 0 {
                let p = self.poll.unwrap();
                self.ready(p, event.token(), event.kind());
                if self.running == false {
                    return 0;
                }
            }
        }
        1
    }
    fn initial(&'a mut self, p: &'a Poll, t: usize, _: usize) {
        self.poll = Some(p);
        self.token = Token(t);
        self.tele = Some(Tele::new(Token(t)));
        self.register(p);
        println!("Welcome to O-CPS Interpreter v{}!", VERSION.to_string());
        Console::prompt();
    }

    fn finalize(&mut self) {
        println!("Bye!");
    }
}
