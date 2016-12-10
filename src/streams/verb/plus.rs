use streams::interpreter::*;
use commands::ast::AST;
use streams::stream::{self, Error, Poll, Async};

pub struct Plus<'ast> {
    lvalue: AST<'ast>,
    rvalue: AST<'ast>,
}

pub fn new<'ast>(lvalue: AST<'ast>, rvalue: AST<'ast>) -> Plus<'ast> {
    Plus {
        lvalue: lvalue,
        rvalue: rvalue,
    }
}

impl<'ast> Plus<'ast> {
    fn a_a(l: i64, r: i64) -> AST<'ast> {
        AST::Number(l + r)
    }
    fn l_a(l: AST<'ast>, r: AST<'ast>) -> AST<'ast> {
        AST::Number(1)
    }
    fn a_l(l: AST<'ast>, r: AST<'ast>) -> AST<'ast> {
        AST::Number(1)
    }
    fn l_l(l: &[i64], r: &[i64]) -> AST<'ast> {
        AST::Number(1)
    }
}

impl<'ast> Iterator for Plus<'ast> {
    type Item = AST<'ast>;
    fn next(&mut self) -> Option<Self::Item> {
        match (&mut self.lvalue, &mut self.rvalue) {
            (&mut AST::Number(ref l), &mut AST::Number(ref r)) => Some(Self::a_a(*l, *r)),
            _ => None,
        }
    }
}

impl<'a, 'ast> Iterator for &'a Plus<'ast> {
    type Item = AST<'ast>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
