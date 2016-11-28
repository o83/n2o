
use streams::adverb::map::{self, Map};
use streams::into_stream::IntoStream;

#[derive(PartialEq,Debug, Clone)]
pub enum Async<T> {
    Ready(T),
    NotReady,
}


#[derive(Debug)]
pub enum Error {
    RuntimeError,
}

pub type Poll<T> = Result<Async<T>, Error>;

pub trait Stream {
    type Item;
    type Error;

    fn poll(&mut self) -> Poll<Self::Item>;

    fn map<U, F>(self, f: F) -> Map<Self, F>
        where F: FnMut(Self::Item) -> U,
              Self: Sized
    {
        map::new(self, f)
    }

}
