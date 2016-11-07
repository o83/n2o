
// Console Remote Shell I/O Reactor Sample by Anton

// $ rlwrap console

extern crate kernel;
#[macro_use]
extern crate core;
extern crate argparse;

use kernel::io::poll::*;
use kernel::reactors::console::Console;
use std::io::{self, BufReader};
use std::io::prelude::*;
use std::fs::File;
use argparse::{ArgumentParser, StoreTrue, Store};

fn main() {
    let mut init = "".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Console options.");
        ap.refer(&mut init)
            .add_option(&["--init"], Store, "Execute initial file.");
        ap.parse_args_or_exit();
    }

    let mut poll = Poll::new().expect("Failed to create Poll");
    let mut c = Console::new();
    if init != "" {
        let f = File::open(init).unwrap();
        let f = BufReader::new(f);
        c.from_buf(f);
    }
    c.run(&mut poll);
}
