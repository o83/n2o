use commands::ast::{AST, Atom, Value};

pub struct Mul<'ast> {
    lvalue: &'ast AST<'ast>,
    rvalue: &'ast AST<'ast>,
}

pub fn new<'ast>(lvalue: &'ast AST<'ast>, rvalue: &'ast AST<'ast>) -> Mul<'ast> {
    Mul {
        lvalue: lvalue,
        rvalue: rvalue,
    }
}

impl<'ast> Mul<'ast> {
    fn a_a(l: i64, r: i64) -> AST<'ast> {
        AST::Atom(Atom::Value(Value::Number(l * r)))
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
            .map(|(l, r)| l * r)
            .collect();
        AST::Atom(Atom::Value(Value::VecInt(a)))
    }

    fn vf_vf(l: &[f64], r: &[f64]) -> AST<'ast> {
        let a: Vec<f64> = l.iter()
            .zip(r)
            .map(|(l, r)| l * r)
            .collect();
        AST::Atom(Atom::Value(Value::VecFloat(a)))
    }
}

impl<'ast> Iterator for Mul<'ast> {
    type Item = AST<'ast>;
    fn next(&mut self) -> Option<Self::Item> {
        match (self.lvalue, self.rvalue) {
            (&AST::Atom(Atom::Value(Value::Number(l))), &AST::Atom(Atom::Value(Value::Number(r)))) => {
                Some(Self::a_a(l, r))
            }
            (&AST::Atom(Atom::Value(Value::VecFloat(ref l))), &AST::Atom(Atom::Value(Value::VecFloat(ref r)))) => {
                Some(Self::vf_vf(l, r))
            }
            (&AST::Atom(Atom::Value(Value::VecInt(ref l))), &AST::Atom(Atom::Value(Value::VecInt(ref r)))) => {
                Some(Self::l_l(l, r))
            }
            _ => None,
        }
    }
}

impl<'a, 'ast> Iterator for &'a Mul<'ast> {
    type Item = AST<'ast>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
