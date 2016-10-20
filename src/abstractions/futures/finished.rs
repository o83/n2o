use abstractions::futures::future::Future;
use abstractions::poll::{Poll, Async};
use core::marker::PhantomData;

#[must_use = "futures do nothing unless polled"]
pub struct Finished<T, E> {
    t: Option<T>,
    _e: PhantomData<E>,
}

pub fn new<T, E>(t: T) -> Finished<T, E> {
    Finished {
        t: Some(t),
        _e: PhantomData,
    }
}

impl<T, E> Future for Finished<T, E> {
    type Item = T;
    type Error = E;


    fn poll(&mut self) -> Poll<T, E> {
        Ok(Async::Ready(self.t.take().expect("cannot poll Finished twice")))
    }
}
