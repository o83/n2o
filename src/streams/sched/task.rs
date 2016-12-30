
#[derive(Debug)]
pub enum Poll<T> {
    Yield(T),
    End,
}

#[derive(Debug)]
pub enum Error {
    RuntimeError,
}

pub trait Task {
    type Item;
    type Error;
    fn poll(&mut self, i: Self::Item) -> Result<Poll<Self::Item>, Error>;
}