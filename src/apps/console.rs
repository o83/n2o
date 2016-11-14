
// Console Remote Shell I/O Reactor Sample by Anton

// $ rlwrap console

#![allow(unused_must_use)]
#![allow(unused_variables)]
#![allow(unused_imports)]

extern crate kernel;
#[macro_use]
extern crate core;
#[macro_use]
extern crate log;

use kernel::io::poll::*;
use kernel::reactors::console::*;
use std::boxed::*;
use std::io::{self, BufReader};
use std::fs::File;
use kernel::args::*;

fn main() {
    let mut poll = Poll::new().expect("Failed to create Poll");
    let mut c = Console::new();

    let mut p = Parser::new();
    if let Ok(init) = p.get("init", true) {
        match File::open(init.unwrap()) {
            Ok(f) => {
                let f = BufReader::new(f);
                c.from_buf(f);
            }
            Err(e) => error!("Error loading init file: {:?}", e),
        }
    }
    if let Ok(help) = p.get("help", false) {
        println!("help: use 'server init <filename>' to boot.");
        return;
    }

    c.run(&mut poll);
}
