#[derive(PartialEq,Debug, Clone)]
pub enum Async<T> {
    Ready(T),
    NotReady,
}

impl<T> Async<T> {
    #[inline]
    pub fn unwrap(self) -> T {
    match self {
         Async::Ready(val) => val,
            Async::NotReady => panic!("called `Async::unwrap()` on a `None` value"),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    NotImplemented,
    RuntimeError,
}

pub type Poll<T> = Result<Async<T>, Error>;