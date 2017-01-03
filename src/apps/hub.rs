extern crate kernel;
use kernel::reactors::hub::Hub;
use kernel::ptr::handle;

fn main() {
    let mut h = handle::new(Hub::new());
    h.borrow_mut().init();
    h.borrow_mut().boil();
}