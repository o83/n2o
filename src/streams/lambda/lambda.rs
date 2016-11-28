use streams::adverb::stream::*;
use commands::ast::{self, Value, List, AST};

pub struct Lambda {
    args: List,
    body: AST,
}

pub fn new(args: List, body: AST) -> Lambda {
    Lambda {
        args: args,
        body: body,
    }
}

impl Iterator for Lambda {
    type Item = Poll<Value>;
    fn next(&mut self) -> Option<Self::Item> {
        Some(Ok(Async::Ready(Value::Integer(123))))
    }
}

impl<'a> Iterator for &'a Lambda {
    type Item = Poll<Value>;
    fn next(&mut self) -> Option<Self::Item> {
        let offset = Value::Integer(123);
        Some(Ok(Async::Ready(offset)))
    }
}
