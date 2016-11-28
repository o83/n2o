// The map steram combinator.

use commands::ast::*;
use streams::stream::{Async, Stream, Poll};
use streams::into_stream::IntoStream;

pub struct Map<S, F> {
    stream: S,
    f: F,
}

pub fn new<S, F, U>(s: S, f: F) -> Map<S, F>
    where S: Stream,
          F: FnMut(S::Item) -> U
{
    Map { stream: s, f: f }
}

impl<S, F, U> Stream for Map<S, F>
    where S: Stream,
          F: FnMut(S::Item) -> U
{
    type Item = U;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Self::Item> {
        match self.stream.poll() {
            Ok(Async::Ready(i)) => Ok(Async::Ready((&mut self.f)(i))),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(e) => Err(e),
        }
    }
}
