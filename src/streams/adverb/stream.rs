// Streams inspired by Iterators.

#[derive(Debug)]
pub enum Async<T> {
    Ready(T),
    NotReady,
}

impl<T> Async<T> {
    #[inline]
    pub fn unwrap(self) -> T {
        match self {
            Async::Ready(val) => val,
            Async::NotReady => panic!("called `Async::unwrap()` on a `NotReady` value"),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    InternalError,
}


pub type Poll<T> = Result<Async<T>, Error>;
