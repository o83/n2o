use commands::ast::Value;
use commands::ast::{Atom, AST};

pub struct Dot<'ast> {
    lvalue: &'ast AST<'ast>,
    rvalue: &'ast AST<'ast>,
}

pub fn new<'ast>(lvalue: &'ast AST<'ast>, rvalue: &'ast AST<'ast>) -> Dot<'ast> {
    Dot {
        lvalue: lvalue,
        rvalue: rvalue,
    }
}

impl<'ast> Dot<'ast> {
    fn a_a(l: i64, r: i64) -> AST<'ast> {
        AST::Atom(Atom::Value(Value::Number(if r == l { 1 } else { 0 })))
    }
    fn l_a(l: &'ast Atom<'ast>, r: &'ast Atom<'ast>) -> AST<'ast> {
        AST::Atom(Atom::Value(Value::Number(1)))
    }
    fn a_l(l: &'ast Atom<'ast>, r: &'ast Atom<'ast>) -> AST<'ast> {
        AST::Atom(Atom::Value(Value::Number(1)))
    }
    fn l_l(l: &[i64], r: &[i64]) -> AST<'ast> {
        AST::Atom(Atom::Value(Value::Number(1)))
    }
}

impl<'ast> Iterator for Dot<'ast> {
    type Item = AST<'ast>;
    fn next(&mut self) -> Option<Self::Item> {
        match (self.lvalue, self.rvalue) {
            (&AST::Atom(Atom::Value(Value::Number(l))), &AST::Atom(Atom::Value(Value::Number(r)))) => {
                Some(AST::Atom(Atom::Value(Value::Float((l + r) as f64))))
            } // TODO: Fix float conversion
            _ => None,
        }
    }
}

impl<'a, 'ast> Iterator for &'a Dot<'ast> {
    type Item = AST<'ast>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
