

use abstractions::poll::Poll;
use abstractions::streams::stream::IntoFuture;
use abstractions::futures::{map, map_err, then, and_then, or_else, into_stream};
use abstractions::futures::into_stream::IntoStream;
use abstractions::futures::map::Map;
use abstractions::futures::map_err::MapErr;
use abstractions::futures::or_else::OrElse;
use abstractions::futures::and_then::AndThen;
use abstractions::futures::then::Then;
use abstractions::tasks::task;

use std::boxed::Box;

pub type BoxFuture<T, E> = Box<Future<Item = T, Error = E> + Send>;

impl<F: ?Sized + Future> Future for Box<F> {
    type Item = F::Item;
    type Error = F::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        (**self).poll()
    }
}

pub trait Future {
    type Item;
    type Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error>;

    fn wait(self) -> Result<Self::Item, Self::Error>
        where Self: Sized
    {
        task::spawn(self).wait_future()
    }

    fn boxed(self) -> BoxFuture<Self::Item, Self::Error>
        where Self: Sized + Send + 'static
    {
        Box::new(self)
    }

    fn map<F, U>(self, f: F) -> Map<Self, F>
        where F: FnOnce(Self::Item) -> U,
              Self: Sized
    {
        assert_future::<U, Self::Error, _>(map::new(self, f))
    }

    fn map_err<F, E>(self, f: F) -> MapErr<Self, F>
        where F: FnOnce(Self::Error) -> E,
              Self: Sized,
    {
        assert_future::<Self::Item, E, _>(map_err::new(self, f))
    }


    fn then<F, B>(self, f: F) -> Then<Self, B, F>
        where F: FnOnce(Result<Self::Item, Self::Error>) -> B,
              B: IntoFuture,
              Self: Sized
    {
        assert_future::<B::Item, B::Error, _>(then::new(self, f))
    }

    fn and_then<F, B>(self, f: F) -> AndThen<Self, B, F>
        where F: FnOnce(Self::Item) -> B,
              B: IntoFuture<Error = Self::Error>,
              Self: Sized
    {
        assert_future::<B::Item, Self::Error, _>(and_then::new(self, f))
    }

    fn or_else<F, B>(self, f: F) -> OrElse<Self, B, F>
        where F: FnOnce(Self::Error) -> B,
              B: IntoFuture<Item = Self::Item>,
              Self: Sized
    {
        assert_future::<Self::Item, B::Error, _>(or_else::new(self, f))
    }

    fn into_stream(self) -> IntoStream<Self>
        where Self: Sized
    {
        into_stream::new(self)
    }
}

impl<'a, F: ?Sized + Future> Future for &'a mut F {
    type Item = F::Item;
    type Error = F::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        (**self).poll()
    }
}

fn assert_future<A, B, F>(t: F) -> F
    where F: Future<Item = A, Error = B>
{
    t
}
