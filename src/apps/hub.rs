extern crate kernel;
use kernel::reactors::hub::Hub;
use kernel::ptr::handle;

fn main() {
    let mut h = handle::new(Hub::new());
    let input1 = "1+2";
    let input2 = "4+5";
    h.borrow_mut().exec(Some(&input1));
    h.borrow_mut().exec(Some(&input2));
    h.borrow_mut().boil();
}