extern crate kernel;

use kernel::reactors::fusion::*;

fn main() {
    let f = Fusion::new();
    let it = f.into_iter();
    for i in it {
        // iterate mixed stream
        let v: &Return = unsafe { &*i };
        println!("Next: {:?}", v);
    }
}