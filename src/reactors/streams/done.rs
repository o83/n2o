// Tail of Stream chain.
//
// done.rs
//
use super::stream::{Async, Stream, Poll};

pub struct Done<T, E> {
    inner: Result<Option<Async<T>>, E>,
}

pub fn done<T, E>(r: Result<T, E>) -> Done<T, E> {
    match r {
        Ok(o) => Done { inner: Ok(Some(Async::Ready(o))) },
        Err(e) => Done { inner: Err(e) },
    }
}

impl<T, E> Stream for Done<T, E> {
    type Item = T;
    type Error = E;

    fn poll(&mut self) -> Poll<T, E> {
        self.inner.clone()
    }
}
