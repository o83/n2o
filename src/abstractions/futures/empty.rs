
use abstractions::futures::future::Future;
use abstractions::poll::{Poll, Async};
use core::marker::PhantomData;

#[must_use = "futures do nothing unless polled"]
pub struct Empty<T, E> {
    _data: PhantomData<(T, E)>,
}

pub fn new<T, E>() -> Empty<T, E> {
    Empty { _data: PhantomData }
}

impl<T, E> Future for Empty<T, E> {
    type Item = T;
    type Error = E;

    fn poll(&mut self) -> Poll<T, E> {
        Ok(Async::NotReady)
    }
}
