extern crate kernel;
use kernel::reactors::hub::Hub;
use kernel::ptr::handle;

fn main() {
    let mut h = handle::new(Hub::new());
    let input = "1+2";
    h.borrow_mut().init(Some(&input));
    h.borrow_mut().boil();
}