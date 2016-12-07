use streams::stream::*;
use commands::ast::{self, AST};
use streams::interpreter::*;

pub struct Lambda {
    args: AST,
    body: AST,
}

pub fn new(args: AST, body: AST) -> Lambda {
    Lambda {
        args: args,
        body: body,
    }
}

impl Iterator for Lambda {
    type Item = Poll<AST>;
    fn next(&mut self) -> Option<Self::Item> {
        Some(Ok(Async::Ready(AST::Number(123))))
    }
}

impl<'a> Iterator for &'a Lambda {
    type Item = Poll<AST>;
    fn next(&mut self) -> Option<Self::Item> {
        let offset = AST::Number(123);
        Some(Ok(Async::Ready(offset)))
    }
}
