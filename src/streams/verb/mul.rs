use commands::ast::Value;
use commands::ast::AST;

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
        AST::Value(Value::Number(l * r))
    }
    fn l_a(l: &'ast AST<'ast>, r: &'ast AST<'ast>) -> AST<'ast> {
        AST::Value(Value::Number(1))
    }
    fn a_l(l: &'ast AST<'ast>, r: &'ast AST<'ast>) -> AST<'ast> {
        AST::Value(Value::Number(1))
    }

    fn l_l(l: &[i64], r: &[i64]) -> AST<'ast> {
        let a: Vec<i64> = l.iter()
            .zip(r)
            .map(|(l, r)| l * r)
            .collect();
        AST::Value(Value::VecInt(a))
    }

    fn vf_vf(l: &[f64], r: &[f64]) -> AST<'ast> {
        let a: Vec<f64> = l.iter()
            .zip(r)
            .map(|(l, r)| l * r)
            .collect();
        println!("VF: {:?}", a);
        AST::Value(Value::VecFloat(a))
    }
}

impl<'ast> Iterator for Mul<'ast> {
    type Item = AST<'ast>;
    fn next(&mut self) -> Option<Self::Item> {
        match (self.lvalue, self.rvalue) {
            (&AST::Value(Value::Number(l)), &AST::Value(Value::Number(r))) => Some(Self::a_a(l, r)),
            (&AST::Value(Value::VecFloat(ref l)), &AST::Value(Value::VecFloat(ref r))) => Some(Self::vf_vf(l, r)),
            (&AST::Value(Value::VecInt(ref l)), &AST::Value(Value::VecInt(ref r))) => Some(Self::l_l(l, r)),
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
