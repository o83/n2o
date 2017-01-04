extern crate kernel;
use kernel::reactors::hub::Hub;
use kernel::ptr::handle;

fn main() {
    let mut h = handle::new(Hub::new());
    let input1 = "a:{snd[0;42]; snd[0;44]; rcv 1; rcv 1;a 1};a 1";
    let input2 = "b:{rcv 0; rcv 0; snd[1;41]; snd[1;43]; b 1};b 1";
    h.borrow_mut().exec(Some(&input1));
    h.borrow_mut().exec(Some(&input2));
    h.borrow_mut().boil();
}