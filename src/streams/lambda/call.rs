use streams::adverb::stream::*;
use streams::lambda::lambda::Lambda;
use commands::ast::{self, Value, List, AST};

pub struct Call {
    callee: Lambda,
    args: List,
}

pub fn new(callee: Lambda, args: List) -> Call {
    Call {
        callee: callee,
        args: args,
    }
}

impl Iterator for Call {
    type Item = Poll<Value>;
    fn next(&mut self) -> Option<Self::Item> {
        Some(Ok(Async::Ready(Value::Integer(123))))
    }
}

impl<'a> Iterator for &'a Call {
    type Item = Poll<Value>;
    fn next(&mut self) -> Option<Self::Item> {
        let res = Value::Integer(123);
        Some(Ok(Async::Ready(res)))
    }
}
