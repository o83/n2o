extern crate kernel;
use kernel::reactors::boot::reactor::{Async, Core};
use kernel::reactors::boot::console::Console;
use std::io::Read;

fn main() {
    let mut c = Core::new();
    let mut o = Box::new(Console::new());
    c.spawn(o);
    for a in c {
        match a {
            Async::Ready((i, s)) => {
                println!("Async: {:?}", i);
                // c.write(i, s.as_bytes());
            }
            x => println!("{:?}", x),
        }
    }
}