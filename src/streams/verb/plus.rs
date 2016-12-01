// An add verb combinator.

use streams::interpreter::*;
use commands::ast::AST;
use streams::stream::{self, Error, Poll, Async};

pub struct Plus {
    lvalue: AST,
    rvalue: AST,
}

pub fn new(lvalue: AST, rvalue: AST) -> Plus {
    Plus {
        lvalue: lvalue,
        rvalue: rvalue,
    }
}

impl Plus {
    // Now just returning simple int
    fn a_a(l: u64, r: u64) -> AST {
        // Need to figure out what integers we have (signed or unsigned)
        AST::Number(l as u64 + r as u64)
    }
    fn l_a(l: AST, r: AST) -> AST {
        AST::Number(1)
    }
    fn a_l(l: AST, r: AST) -> AST {
        AST::Number(1)
    }
    fn l_l(l: &[u64], r: &[u64]) -> AST {
        AST::Number(1)
    }
}

impl Iterator for Plus {
    type Item = Poll<AST>;

    fn next(&mut self) -> Option<Self::Item> {
        match (&mut self.lvalue, &mut self.rvalue) {
            (&mut AST::Number(ref l), &mut AST::Number(ref r)) => {
                Some(Ok(Async::Ready(Self::a_a(*l, *r))))
            }
            (&mut AST::List(ref l), &mut AST::List(ref r)) => {
                // Here we need to get Vec from Box and pass to l_l()
                // Some(Ok(Async::Ready(Self::l_l(l.borrow(), r.borrow()))))
                Some(Err(Error::NotImplemented))

            }
            _ => Some(Err(Error::NotImplemented)),
        }
    }
}

impl<'a> Iterator for &'a Plus {
    type Item = Poll<AST>;

    fn next(&mut self) -> Option<Self::Item> {
        // not implemented yet
        None
    }
}
