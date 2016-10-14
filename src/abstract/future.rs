#![no_std]
#[macro_use]
#[cfg(feature = "use_std")]
extern crate std;

#[macro_use]
extern crate log;

macro_rules! if_std {
    ($($i:item)*) => ($(
        #[cfg(feature = "use_std")]
        $i
    )*)
}

#[macro_use]
mod poll;
pub use poll::{Poll, Async};

mod done;
mod empty;
mod failed;
mod finished;
mod lazy;
pub use done::{done, Done};
pub use empty::{empty, Empty};
pub use failed::{failed, Failed};
pub use finished::{finished, Finished};
pub use lazy::{lazy, Lazy};

mod into_stream;
mod join;
mod map;
mod map_err;
mod or_else;
mod select;
mod then;
pub use into_stream::IntoStream;
pub use map::Map;
pub use or_else::OrElse;
pub use then::Then;

if_std! {
    mod lock;
    mod slot;
    pub mod task;

    mod catch_unwind;
    mod collect;
    mod oneshot;
    mod select_all;
    pub use catch_unwind::CatchUnwind;
    pub use collect::{collect, Collect};
    pub use oneshot::{oneshot, Oneshot, Complete, Canceled};
    pub use select_all::{SelectAll, SelectAllNext, select_all};

    /// A type alias for `Box<Future + Send>`
    pub type BoxFuture<T, E> = std::boxed::Box<Future<Item = T, Error = E> + Send>;

    impl<F: ?Sized + Future> Future for std::boxed::Box<F> {
        type Item = F::Item;
        type Error = F::Error;

        fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
            (**self).poll()
        }
    }
}

// streams
pub mod stream;

// impl details
mod chain;

pub trait Future {
    type Item;
    type Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error>;

    #[cfg(feature = "use_std")]
    fn wait(self) -> Result<Self::Item, Self::Error>
        where Self: Sized
    {
        task::spawn(self).wait_future()
    }

    #[cfg(feature = "use_std")]
    fn boxed(self) -> BoxFuture<Self::Item, Self::Error>
        where Self: Sized + Send + 'static
    {
        ::std::boxed::Box::new(self)
    }

    fn map<F, U>(self, f: F) -> Map<Self, F>
        where F: FnOnce(Self::Item) -> U,
              Self: Sized,
    {
        assert_future::<U, Self::Error, _>(map::new(self, f))
    }

    fn then<F, B>(self, f: F) -> Then<Self, B, F>
        where F: FnOnce(Result<Self::Item, Self::Error>) -> B,
              B: IntoFuture,
              Self: Sized,
    {
        assert_future::<B::Item, B::Error, _>(then::new(self, f))
    }

    fn and_then<F, B>(self, f: F) -> AndThen<Self, B, F>
        where F: FnOnce(Self::Item) -> B,
              B: IntoFuture<Error = Self::Error>,
              Self: Sized,
    {
        assert_future::<B::Item, Self::Error, _>(and_then::new(self, f))
    }

    fn or_else<F, B>(self, f: F) -> OrElse<Self, B, F>
        where F: FnOnce(Self::Error) -> B,
              B: IntoFuture<Item = Self::Item>,
              Self: Sized,
    {
        assert_future::<Self::Item, B::Error, _>(or_else::new(self, f))
    }

    fn into_stream(self) -> IntoStream<Self>
        where Self: Sized
    {
        into_stream::new(self)
    }

    fn fuse(self) -> Fuse<Self>
        where Self: Sized
    {
        let f = fuse::new(self);
        assert_future::<Self::Item, Self::Error, _>(f)
    }

    #[cfg(feature = "use_std")]
    fn catch_unwind(self) -> CatchUnwind<Self>
        where Self: Sized + std::panic::UnwindSafe
    {
        catch_unwind::new(self)
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
    where F: Future<Item=A, Error=B>,
{
    t
}

/// Class of types which can be converted themselves into a future.
pub trait IntoFuture {
    type Future: Future<Item=Self::Item, Error=Self::Error>;
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
