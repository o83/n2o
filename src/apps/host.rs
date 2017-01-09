extern crate kernel;
use kernel::reactors::host::Host;

fn main() {
    let mut h = Host::new();
    h.run();
}