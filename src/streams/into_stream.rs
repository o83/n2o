
use streams::stream::Stream;
use streams::vectors::done::{self, Done};

pub trait IntoStream {
    type Item;
    type Error;
    type Stream: Stream<Item = Self::Item, Error = Self::Error>;

    fn into_stream(self) -> Self::Stream;
}

impl<S: Stream> IntoStream for S {
    type Stream = S;
    type Item = S::Item;
    type Error = S::Error;

    fn into_stream(self) -> S {
        self
    }
}

impl<T, E> IntoStream for Result<T, E> {
    type Stream = Done<T, E>;
    type Item = T;
    type Error = E;

    fn into_stream(self) -> Done<T, E> {
        done::new(self)
    }
}
