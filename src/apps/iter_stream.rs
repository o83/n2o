extern crate kernel;
use kernel::reactors::streams::iter::get;
use kernel::reactors::streams::iter::stream::Async;

fn main() {
    // Create messages stream
    let mut s = get::new(0, 1);

    // Just get one item from it
    println!("Next: {:?}", s.next());

    // Take is useful for tasks with priorities
    for i in (&s).take(5) {
        println!("Taken: {:?}", i);
    }

    // Map usage:
    // let mut i = s.into_iter().map(|x| println!("Message: {:?}", x));
    // i.next();

    // Filter:
    let mut f = s.take(4).filter(|x| {
        match *x {
            Ok(ref v) => {
                match v {
                    &Async::Ready(v) => v < 3,
                    _ => false,
                }
            }
            Err(_) => false,
        }
    });
    println!("Filter: {:?}", f.next());
    println!("Filter: {:?}", f.next());
    println!("Filter: {:?}", f.next());
    println!("Filter: {:?}", f.next());
}
