extern crate kernel;

// The Kernel Shell

use std::io::prelude::*;
use std::io;
use kernel::commands::*;

pub fn prompt() {
    print!("> ");
    let _ = io::stdout().flush();
}

fn main() {
    let mut input_line = String::new();
    loop {
        prompt();
        io::stdin().read_line(&mut input_line).ok().expect("The read line failed");
        match input_line.trim() {
            "exit" => break,
            line => {
                println!("{:?}", command::parse_Mex(&line.to_string()));
            }
        }
        input_line.clear();
    }
    println!("May the force be with you!");
}
