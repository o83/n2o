// An add verb combinator.

use streams::interpreter::{self, Value, List};
use commands::ast::AST;
use streams::stream::{self, Error, Poll, Async};

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

impl Add {
    // Now just returning simple int
    fn a_a(&mut self) -> Value {
        Value::Integer(1)
    }
    fn l_a(&mut self) -> Value {
        Value::Integer(1)
    }
    fn l_l(&mut self) -> Value {
        Value::Integer(1)
    }
    fn a_l(&mut self) -> Value {
        Value::Integer(1)
    }
}

impl Iterator for Add {
    type Item = Poll<Value>;

    fn next(&mut self) -> Option<Self::Item> {
        let lr = (&mut self.lvalue, &mut self.rvalue);
        match lr {
            (&mut AST::Number(l), &mut AST::Number(ref r)) => {
                Some(Ok(Async::Ready(Value::Integer(1))))
            }
            (&mut AST::Number(l), &mut AST::List(ref r)) => {
                Some(Ok(Async::Ready(Value::Integer(2))))
            }
            _ => Some(Err(Error::NotImplemented)),
        }
    }
}

impl<'a> Iterator for &'a Add {
    type Item = Poll<Value>;

    fn next(&mut self) -> Option<Self::Item> {
        // not implemented yet
        None
    }
}
