use abstractions::futures::future::Future;
use abstractions::poll::Poll;
use abstractions::futures::done::{done, Done};
use abstractions::streams::flatten::Flatten;
use abstractions::streams::skip::Skip;
use abstractions::streams::filter::Filter;
use abstractions::streams::filter_map::FilterMap;
use abstractions::streams::fold::Fold;
use abstractions::streams::map::Map;
use abstractions::streams::map_err::MapErr;
use abstractions::streams::then::Then;
use abstractions::streams::or_else::OrElse;
use abstractions::streams::and_then::AndThen;
use abstractions::streams::take::Take;
use abstractions::streams::wait::Wait;
use abstractions::streams::collect::Collect;
use abstractions::streams::{future, skip, flatten, map, map_err, collect, take, fold, filter,
                            filter_map, or_else, and_then, then, wait};
use abstractions::streams::future::StreamFuture;

macro_rules! if_std {
    ($($i:item)*) => ($(
        #[cfg(feature = "use_std")]
        $i
    )*)
}

// if_std! {
use std;

pub type BoxStream<T, E> = ::std::boxed::Box<Stream<Item = T, Error = E> + Send>;

impl<S: ?Sized + Stream> Stream for ::std::boxed::Box<S> {
    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        (**self).poll()
    }
}
// }

pub trait Stream {
    type Item;
    type Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error>;

    fn wait(self) -> Wait<Self>
        where Self: Sized
    {
        wait::new(self)
    }

    fn boxed(self) -> BoxStream<Self::Item, Self::Error>
        where Self: Sized + Send + 'static
    {
        ::std::boxed::Box::new(self)
    }

    fn into_future(self) -> StreamFuture<Self>
        where Self: Sized
    {
        future::new(self)
    }

    fn collect(self) -> Collect<Self>
        where Self: Sized
    {
        collect::new(self)
    }

    fn map<U, F>(self, f: F) -> Map<Self, F>
        where F: FnMut(Self::Item) -> U,
              Self: Sized
    {
        map::new(self, f)
    }

    fn map_err<U, F>(self, f: F) -> MapErr<Self, F>
        where F: FnMut(Self::Error) -> U,
              Self: Sized
    {
        map_err::new(self, f)
    }

    fn flatten(self) -> Flatten<Self>
        where Self::Item: Stream,
              <Self::Item as Stream>::Error: From<Self::Error>,
              Self: Sized
    {
        flatten::new(self)
    }

    fn filter<F>(self, f: F) -> Filter<Self, F>
        where F: FnMut(&Self::Item) -> bool,
              Self: Sized
    {
        filter::new(self, f)
    }

    fn filter_map<F, B>(self, f: F) -> FilterMap<Self, F>
        where F: FnMut(Self::Item) -> Option<B>,
              Self: Sized
    {
        filter_map::new(self, f)
    }

    fn then<F, U>(self, f: F) -> Then<Self, F, U>
        where F: FnMut(Result<Self::Item, Self::Error>) -> U,
              U: IntoFuture,
              Self: Sized
    {
        then::new(self, f)
    }

    fn and_then<F, U>(self, f: F) -> AndThen<Self, F, U>
        where F: FnMut(Self::Item) -> U,
              U: IntoFuture<Error = Self::Error>,
              Self: Sized
    {
        and_then::new(self, f)
    }

    fn or_else<F, U>(self, f: F) -> OrElse<Self, F, U>
        where F: FnMut(Self::Error) -> U,
              U: IntoFuture<Item = Self::Item>,
              Self: Sized
    {
        or_else::new(self, f)
    }

    fn fold<F, T, Fut>(self, init: T, f: F) -> Fold<Self, F, Fut, T>
        where F: FnMut(T, Self::Item) -> Fut,
              Fut: IntoFuture<Item = T>,
              Self::Error: From<Fut::Error>,
              Self: Sized
    {
        fold::new(self, f, init)
    }

    fn skip(self, amt: u64) -> Skip<Self>
        where Self: Sized
    {
        skip::new(self, amt)
    }

    fn take(self, amt: u64) -> Take<Self>
        where Self: Sized
    {
        take::new(self, amt)
    }

    #[cfg(feature = "use_std")]
    fn buffered(self, amt: usize) -> Buffered<Self>
        where Self::Item: IntoFuture<Error = <Self as Stream>::Error>,
              Self: Sized
    {
        buffered::new(self, amt)
    }

    #[cfg(feature = "use_std")]
    fn buffer_unordered(self, amt: usize) -> BufferUnordered<Self>
        where Self::Item: IntoFuture<Error = <Self as Stream>::Error>,
              Self: Sized
    {
        buffer_unordered::new(self, amt)
    }
}

impl<'a, S: ?Sized + Stream> Stream for &'a mut S {
    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        (**self).poll()
    }
}

/// Class of types which can be converted themselves into a future.
pub trait IntoFuture {
    type Future: Future<Item = Self::Item, Error = Self::Error>;
    type Item;
    type Error;
    fn into_future(self) -> Self::Future;
}

impl<F: Future> IntoFuture for F {
    type Future = F;
    type Item = F::Item;
    type Error = F::Error;

    fn into_future(self) -> F {
        self
    }
}

impl<T, E> IntoFuture for Result<T, E> {
    type Future = Done<T, E>;
    type Item = T;
    type Error = E;

    fn into_future(self) -> Done<T, E> {
        done(self)
    }
}
