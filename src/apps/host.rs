extern crate kernel;
use kernel::reactors::init;

fn main() {
    let mut h = init::host();
    h.run();
}