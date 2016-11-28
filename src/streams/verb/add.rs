// An add verb combinator.

use streams::vectors::stream::*;
use commands::ast::AST;

pub struct Add {
    lvalue: AST,
    rvalue: AST,
}

pub fn new(lvalue: AST, rvalue: AST) -> Add {
    Add {
        lvalue: lvalue,
        rvalue: rvalue,
    }
}

impl Iterator for Add {
    type Item = Poll<AST>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Ok(Async::Ready(AST::Number(1))))
    }
}

impl<'a> Iterator for &'a Add {
    type Item = Poll<AST>;

    fn next(&mut self) -> Option<Self::Item> {
        // not implemented yet
        None
    }
}