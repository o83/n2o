extern crate kernel;

use kernel::abstractions::futures::future::Future;
// use kernel::abstractions::futures::done::{done};
use kernel::abstractions::poll::Async;
use kernel::abstractions::queues::channel;
// use kernel::abstractions::queues::channel::{Sender};
use kernel::abstractions::tasks::task;
use std::sync::Arc;

mod support;
use support::*;


#[test]
fn drop_sender() {
    let (tx, mut rx) = channel::create::<u32, u32>();
    drop(tx);
    sassert_done(&mut rx);
}

#[test]
fn drop_rx() {
    let (tx, rx) = channel::create::<u32, u32>();
    let tx = tx.send(Ok(1)).wait().ok().unwrap();
    drop(rx);
    assert!(tx.send(Ok(1)).wait().is_err());
}

struct Unpark;

impl task::Unpark for Unpark {
    fn unpark(&self) {}
}

#[test]
fn poll_future_then_drop() {
    let (tx, _rx) = channel::create::<u32, u32>();

    let tx = tx.send(Ok(1));
    let mut t = task::spawn(tx);

    // First poll succeeds
    let tx = match t.poll_future(Arc::new(Unpark)) {
        Ok(Async::Ready(tx)) => tx,
        _ => panic!(),
    };

    // Send another value
    let tx = tx.send(Ok(2));
    let mut t = task::spawn(tx);

    // Second poll doesn't
    match t.poll_future(Arc::new(Unpark)) {
        Ok(Async::NotReady) => {}
        _ => panic!(),
    };

    drop(t);
}
