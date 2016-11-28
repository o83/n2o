
use streams::stream::Stream;

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

