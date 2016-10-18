use core::mem;

use abstractions::futures::future::Future;
use abstractions::streams::stream::IntoFuture;
use abstractions::poll::Poll;

#[must_use = "futures do nothing unless polled"]
pub struct Lazy<F, R: IntoFuture> {
    inner: _Lazy<F, R::Future>,
}

enum _Lazy<F, R> {
    First(F),
    Second(R),
    Moved,
}

pub fn lazy<F, R>(f: F) -> Lazy<F, R>
    where F: FnOnce() -> R,
          R: IntoFuture
{
    Lazy { inner: _Lazy::First(f) }
}

impl<F, R> Lazy<F, R>
    where F: FnOnce() -> R,
          R: IntoFuture
{
    fn get(&mut self) -> &mut R::Future {
        match self.inner {
            _Lazy::First(_) => {}
            _Lazy::Second(ref mut f) => return f,
            _Lazy::Moved => panic!(), // can only happen if `f()` panics
        }
        match mem::replace(&mut self.inner, _Lazy::Moved) {
            _Lazy::First(f) => self.inner = _Lazy::Second(f().into_future()),
            _ => panic!(), // we already found First
        }
        match self.inner {
            _Lazy::Second(ref mut f) => f,
            _ => panic!(), // we just stored Second
        }
    }
}

impl<F, R> Future for Lazy<F, R>
    where F: FnOnce() -> R,
          R: IntoFuture
{
    type Item = R::Item;
    type Error = R::Error;

    fn poll(&mut self) -> Poll<R::Item, R::Error> {
        self.get().poll()
    }
}
