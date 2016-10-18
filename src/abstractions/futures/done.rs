
use abstractions::futures::future::Future;
use abstractions::poll::{Poll, Async};

#[must_use = "futures do nothing unless polled"]
pub struct Done<T, E> {
    inner: Option<Result<T, E>>,
}

pub fn done<T, E>(r: Result<T, E>) -> Done<T, E> {
    Done { inner: Some(r) }
}

impl<T, E> Future for Done<T, E> {
    type Item = T;
    type Error = E;

    fn poll(&mut self) -> Poll<T, E> {
        self.inner.take().expect("cannot poll Done twice").map(Async::Ready)
    }
}
