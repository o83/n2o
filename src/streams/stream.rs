
use commands::ast::*;
use streams::adverb::map::{self, Map};
use streams::into_stream::IntoStream;

pub enum Async<T> {
    Ready(T),
    NotReady,
}


pub type Poll<T> = Result<Async<T>, Error>;

//pub type Poll<T, E> = Result<Option<Async<T>>, E>;

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
