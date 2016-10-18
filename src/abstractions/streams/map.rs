
use abstractions::streams::stream::Stream;
use abstractions::poll::{Async, Poll};

#[must_use = "streams do nothing unless polled"]
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

    fn poll(&mut self) -> Poll<Option<U>, S::Error> {
        let option = try_ready!(self.stream.poll());
        Ok(Async::Ready(option.map(&mut self.f)))
    }
}
