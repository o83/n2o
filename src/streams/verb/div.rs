use commands::ast::Value;
use commands::ast::{Atom, AST};

pub struct Div<'ast> {
    lvalue: &'ast AST<'ast>,
    rvalue: &'ast AST<'ast>,
}

pub fn new<'ast>(lvalue: &'ast AST<'ast>, rvalue: &'ast AST<'ast>) -> Div<'ast> {
    Div {
        lvalue: lvalue,
        rvalue: rvalue,
    }
}

impl<'ast> Div<'ast> {
    fn a_a(l: i64, r: i64) -> AST<'ast> {
        AST::Atom(Atom::Value(Value::Number(l / r)))
    }
    fn l_a(l: &'ast Atom<'ast>, r: &'ast Atom<'ast>) -> AST<'ast> {
        AST::Atom(Atom::Value(Value::Number(1)))
    }
    fn a_l(l: &'ast Atom<'ast>, r: &'ast Atom<'ast>) -> AST<'ast> {
        AST::Atom(Atom::Value(Value::Number(1)))
    }
    fn l_l(l: &[i64], r: &[i64]) -> AST<'ast> {
        let a: Vec<i64> = l.iter()
            .zip(r)
            .map(|(l, r)| l / r)
            .collect();
        AST::Atom(Atom::Value(Value::VecInt(a)))
    }
}

impl<'ast> Iterator for Div<'ast> {
    type Item = AST<'ast>;
    fn next(&mut self) -> Option<Self::Item> {
        match (self.lvalue, self.rvalue) {
            (&AST::Atom(Atom::Value(Value::Number(l))), &AST::Atom(Atom::Value(Value::Number(r)))) => {
                Some(Self::a_a(l, r))
            }
            _ => None,
        }
    }
}

impl<'a, 'ast> Iterator for &'a Div<'ast> {
    type Item = AST<'ast>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
