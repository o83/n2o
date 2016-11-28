// An add verb combinator.

use streams::interpreter::{self, Value, List};
use commands::ast::*;
use streams::stream::{self, Poll, Async};

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
    type Item = Poll<Value>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Ok(Async::Ready(Value::Integer(1))))
    }
}

impl<'a> Iterator for &'a Add {
    type Item = Poll<Value>;

    fn next(&mut self) -> Option<Self::Item> {
        // not implemented yet
        None
    }
}