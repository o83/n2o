use abstractions::futures::future::Future;
use abstractions::poll::Poll;
use core::marker::PhantomData;

#[must_use = "futures do nothing unless polled"]
pub struct Failed<T, E> {
    _t: PhantomData<T>,
    e: Option<E>,
}

pub fn failed<T, E>(e: E) -> Failed<T, E> {
    Failed {
        _t: PhantomData,
        e: Some(e),
    }
}

impl<T, E> Future for Failed<T, E> {
    type Item = T;
    type Error = E;

    fn poll(&mut self) -> Poll<T, E> {
        Err(self.e.take().expect("cannot poll Failed twice"))
    }
}
