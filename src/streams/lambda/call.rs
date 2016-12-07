use streams::stream::*;
use streams::lambda::lambda::Lambda;
use commands::ast::{self, AST};
use streams::interpreter::*;

pub struct Call {
    callee: Lambda,
    args: AST,
}

pub fn new(callee: Lambda, args: AST) -> Call {
    Call {
        callee: callee,
        args: args,
    }
}

impl Iterator for Call {
    type Item = Poll<AST>;
    fn next(&mut self) -> Option<Self::Item> {
        Some(Ok(Async::Ready(AST::Number(123))))
    }
}

impl<'a> Iterator for &'a Call {
    type Item = Poll<AST>;
    fn next(&mut self) -> Option<Self::Item> {
        let res = AST::Number(123);
        Some(Ok(Async::Ready(res)))
    }
}
