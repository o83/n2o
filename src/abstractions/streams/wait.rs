use abstractions::streams::stream::Stream;
use abstractions::tasks::task;

#[must_use = "iterators do nothing unless advanced"]
pub struct Wait<S> {
    stream: task::Spawn<S>,
}

pub fn new<S: Stream>(s: S) -> Wait<S> {
    Wait {
        stream: task::spawn(s),
    }
}

impl<S: Stream> Iterator for Wait<S> {
    type Item = Result<S::Item, S::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.stream.wait_stream()
    }
}
