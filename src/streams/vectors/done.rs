
use streams::stream::{Async, Stream, Poll};

pub struct Done<T, E> {
    inner: Option<Result<Option<Async<T>>, E>>,
}

pub fn new<T, E>(r: Result<T, E>) -> Done<T, E> {
    match r {
        Ok(o) => Done { inner: Some(Ok(Some(Async::Ready(o)))) },
        Err(e) => Done { inner: Some(Err(e)) },
    }
}

impl<T, E> Stream for Done<T, E> {
    type Item = T;
    type Error = E;

    fn poll(&mut self) -> Poll<T, E> {
        self.inner.take().expect("cannot poll Done twice")
    }
}
