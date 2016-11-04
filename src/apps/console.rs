
// Console Remote Shell I/O Reactor Sample by Anton

// $ rlwrap console

extern crate kernel;

use kernel::io::poll::*;
use kernel::io::console::Console;

fn main() {
    let mut poll = Poll::new().expect("Failed to create Poll");
    let mut c = Console::new();
    println!("REGISTER: {:?}", c.run(&mut poll));
}
