// Much like rust's Iterator::into_iter: to
// wrap any type by Stream.

use super::stream::Stream;

pub trait IntoStream {
    type Item;
    type Error;
    type Stream: Stream<Item = Self::Item, Error = Self::Error>;

    fn into_stream(self) -> Self::Stream;
}