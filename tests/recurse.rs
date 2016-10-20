extern crate kernel;

use std::sync::mpsc::channel;

use kernel::abstractions::futures::future::Future;
use kernel::abstractions::futures::finished;

#[test]
fn lots() {
    fn doit(n: usize) -> Box<Future<Item = (), Error = ()> + Send> {
        if n == 0 {
            finished::new(()).boxed()
        } else {
            finished::new(n - 1).and_then(doit).boxed()
        }
    }

    let (tx, rx) = channel();
    ::std::thread::spawn(|| doit(1_000).map(move |_| tx.send(()).unwrap()).wait());
    rx.recv().unwrap();
}
