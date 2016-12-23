
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
extern crate time;
extern crate env_logger;

use env_logger::LogBuilder;
use log::{LogRecord, LogLevelFilter};
use std::env;
use kernel::io::poll::*;
use kernel::reactors::console::*;
use std::boxed::*;
use std::io::{self, BufReader};
use std::fs::File;
use kernel::args::*;

fn main() {
    let mut poll = Poll::new().expect("Failed to create Poll");
    let mut c = Console::new();
    setup_logger();
    let mut p = Parser::new();
    if let Ok(init) = p.get("init", true) {
        match File::open(init.unwrap()) {
            Ok(f) => {
                c.read_all(f);
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

// logger initial setup
fn setup_logger() {
    let format = |record: &LogRecord| {
        let t = time::now();
        format!("{}\t[{},{:03} {}:{}]\t--> {}",
                record.level(),
                time::strftime("%Y-%m-%d %H:%M:%S", &t).unwrap(),
                t.tm_nsec / 1000_000,
                record.location().__module_path,
                record.location().__line,
                record.args())
    };

    let mut builder = LogBuilder::new();
    builder.format(format).filter(Some("kernel"), LogLevelFilter::Debug);

    if env::var("RUST_LOG").is_ok() {
        builder.parse(&env::var("RUST_LOG").unwrap());
    }

    builder.init().unwrap();
}
