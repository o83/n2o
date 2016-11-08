
// Console Remote Shell I/O Reactor Sample by Anton

// $ rlwrap console

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
use kernel::args::Parser;

fn main() {
    let mut poll = Poll::new().expect("Failed to create Poll");
    let mut c = Console::new();

    Parser::new()
        .arg("init",
             |x| {
            match File::open(x) {
                Ok(f) => {
                    let f = BufReader::new(f);
                    c.from_buf(f);
                }
                Err(e) => error!("Error loading init file: {:?}", e),
            }
        })
        // .arg("help",
        //      Box::new(|x| {
        //          println!("help: use 'server init <filename>' to boot.");
        //      }))
        .parse();

    c.run(&mut poll);
}
