use core::marker::PhantomData;
use abstractions::streams::stream::Stream;
use abstractions::poll::{Async, Poll};

#[must_use = "streams do nothing unless polled"]
pub struct Empty<T, E> {
    _data: PhantomData<(T, E)>,
}
pub fn empty<T, E>() -> Empty<T, E> {
    Empty { _data: PhantomData }
}

impl<T, E> Stream for Empty<T, E>
    where T: Send + 'static,
          E: Send + 'static
{
    type Item = T;
    type Error = E;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        Ok(Async::Ready(None))
    }
}
