#[macro_use]
extern crate kernel;
use kernel::reactors::init;

fn main() {
    let mut h = init::host();
    h.borrow_mut().run();
}