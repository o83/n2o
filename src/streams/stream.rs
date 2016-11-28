#[derive(PartialEq,Debug, Clone)]
pub enum Async<T> {
    Ready(T),
    NotReady,
}

#[derive(Debug)]
pub enum Error {
    NotImplemented,
    RuntimeError,
}

pub type Poll<T> = Result<Async<T>, Error>;