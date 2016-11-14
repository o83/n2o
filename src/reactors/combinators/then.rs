// Then combinator

use reactors::streams::into_stream::IntoStream;
use reactors::streams::stream::{Async, Stream, Poll};

pub struct Then<S, F, U>
    where U: IntoStream
{
    stream: S,
    chain: Option<U::Stream>,
    f: F,
}

pub fn new<S, F, U>(s: S, f: F) -> Then<S, F, U>
    where S: Stream,
          F: FnMut(Result<S::Item, S::Error>) -> U,
          U: IntoStream
{
    Then {
        stream: s,
        chain: None,
        f: f,
    }
}

impl<S, F, U> Stream for Then<S, F, U>
    where S: Stream,
          F: FnMut(Result<S::Item, S::Error>) -> U,
          U: IntoStream
{
    type Item = U::Item;
    type Error = U::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if self.chain.is_none() {
            let item = match self.stream.poll() {
                Ok(Some(Async::NotReady)) => return Ok(Some(Async::NotReady)),
                Ok(None) => return Ok(None),
                Ok(Some(Async::Ready(e))) => Ok(e),
                Err(e) => Err(e),
            };
            self.chain = Some((self.f)(item).into_stream());
        }
        assert!(self.chain.is_some());
        match self.chain.as_mut().unwrap().poll() {
            Ok(Some(Async::Ready(e))) => {
                self.chain = None;
                Ok(Some(Async::Ready(e)))
            }
            Err(e) => {
                self.chain = None;
                Err(e)
            }
            Ok(Some(Async::NotReady)) => Ok(Some(Async::NotReady)),
            Ok(None) => Ok(None),
        }
    }
}
