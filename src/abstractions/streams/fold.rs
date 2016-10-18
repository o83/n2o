use core::mem;
use abstractions::futures::future::Future;
use abstractions::streams::stream::{IntoFuture, Stream};
use abstractions::poll::{Async, Poll};


#[must_use = "streams do nothing unless polled"]
pub struct Fold<S, F, Fut, T>
    where Fut: IntoFuture
{
    stream: S,
    f: F,
    state: State<T, Fut::Future>,
}

enum State<T, F>
    where F: Future
{
    Empty,
    Ready(T),
    Processing(F),
}

pub fn new<S, F, Fut, T>(s: S, f: F, t: T) -> Fold<S, F, Fut, T>
    where S: Stream,
          F: FnMut(T, S::Item) -> Fut,
          Fut: IntoFuture<Item = T>,
          S::Error: From<Fut::Error>
{
    Fold {
        stream: s,
        f: f,
        state: State::Ready(t),
    }
}

impl<S, F, Fut, T> Future for Fold<S, F, Fut, T>
    where S: Stream,
          F: FnMut(T, S::Item) -> Fut,
          Fut: IntoFuture<Item = T>,
          S::Error: From<Fut::Error>
{
    type Item = T;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<T, S::Error> {
        loop {
            match mem::replace(&mut self.state, State::Empty) {
                State::Empty => panic!("cannot poll Fold twice"),
                State::Ready(state) => {
                    match try!(self.stream.poll()) {
                        Async::Ready(Some(e)) => {
                            let future = (self.f)(state, e);
                            let future = future.into_future();
                            self.state = State::Processing(future);
                        }
                        Async::Ready(None) => return Ok(Async::Ready(state)),
                        Async::NotReady => {
                            self.state = State::Ready(state);
                            return Ok(Async::NotReady);
                        }
                    }
                }
                State::Processing(mut fut) => {
                    match try!(fut.poll()) {
                        Async::Ready(state) => self.state = State::Ready(state),
                        Async::NotReady => {
                            self.state = State::Processing(fut);
                            return Ok(Async::NotReady);
                        }
                    }
                }
            }
        }
    }
}
