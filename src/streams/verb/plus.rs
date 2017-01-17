use commands::ast::Value;
use commands::ast::AST;

pub struct Plus<'ast> {
    lvalue: &'ast AST<'ast>,
    rvalue: &'ast AST<'ast>,
}

pub fn new<'ast>(lvalue: &'ast AST<'ast>, rvalue: &'ast AST<'ast>) -> Plus<'ast> {
    Plus {
        lvalue: lvalue,
        rvalue: rvalue,
    }
}

impl<'ast> Plus<'ast> {
    fn a_a(l: i64, r: i64) -> AST<'ast> {
        AST::Value(Value::Number(l + r))
    }
    fn l_a(l: &'ast AST<'ast>, r: &'ast AST<'ast>) -> AST<'ast> {
        AST::Value(Value::Number(1))
    }
    fn a_l(l: &'ast AST<'ast>, r: &'ast AST<'ast>) -> AST<'ast> {
        AST::Value(Value::Number(1))
    }

    fn v_v(l: &[i64], r: &[i64]) -> AST<'ast> {
        let a:Vec<i64> = l.iter().zip(r)
            .map(|(l,r)| l+r)
            .collect();
        AST::Value(Value::VecInt(a))
    }

    fn v_a(l: &[i64], r: i64) -> AST<'ast> {
        let a:Vec<i64> = l.iter()
            .map(|x| x+r)
            .collect();
        AST::Value(Value::VecInt(a))
    }
}

impl<'ast> Iterator for Plus<'ast> {
    type Item = AST<'ast>;
    fn next(&mut self) -> Option<Self::Item> {
        match (self.lvalue, self.rvalue) {
            (&AST::Value(Value::Number(l)), &AST::Value(Value::Number(r))) => Some(Self::a_a(l, r)),
            (&AST::Value(Value::VecInt(ref l)), &AST::Value(Value::VecInt(ref r))) => Some(Self::v_v(l, r)),
            (&AST::Value(Value::Number(l)), &AST::Value(Value::VecInt(ref r))) => Some(Self::v_a(r, l)),
            (&AST::Value(Value::VecInt(ref r)), &AST::Value(Value::Number(l))) => Some(Self::v_a(r, l)),
            _ => None
        }
    }
}

impl<'a, 'ast> Iterator for &'a Plus<'ast> {
    type Item = AST<'ast>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
