
// Console Remote Shell I/O Reactor Sample by Anton

// $ rlwrap console

extern crate kernel;
#[macro_use]
extern crate core;
#[macro_use]
extern crate log;

use kernel::io::poll::*;
use kernel::reactors::console::Console;
use std::io::{self, BufReader};
use std::fs::File;
use kernel::util::argparse::ArgParser;

fn main() {
    let mut poll = Poll::new().expect("Failed to create Poll");
    let mut c = Console::new();

    ArgParser::new()
        .arg("init",
             Box::new(|x| {
            match File::open(x) {
                Ok(f) => {
                    let f = BufReader::new(f);
                    c.from_buf(f);
                }
                Err(e) => error!("Error loading init file: {:?}", e),
            }
        }))
        .arg("help",
             Box::new(|x| {
                 println!("help: this is fucking OS, man!");
             }))
        .parse();

    c.run(&mut poll);
}
