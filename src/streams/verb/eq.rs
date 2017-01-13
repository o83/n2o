use commands::ast::AST;

pub struct Eq<'ast> {
    lvalue: &'ast AST<'ast>,
    rvalue: &'ast AST<'ast>,
}

pub fn new<'ast>(lvalue: &'ast AST<'ast>, rvalue: &'ast AST<'ast>) -> Eq<'ast> {
    Eq {
        lvalue: lvalue,
        rvalue: rvalue,
    }
}

impl<'ast> Eq<'ast> {
    fn a_a(l: i64, r: i64) -> AST<'ast> {
        AST::Number(if r == l { 1 } else { 0 })
    }
    fn l_a(l: &'ast AST<'ast>, r: &'ast AST<'ast>) -> AST<'ast> {
        AST::Number(1)
    }
    fn a_l(l: &'ast AST<'ast>, r: &'ast AST<'ast>) -> AST<'ast> {
        AST::Number(1)
    }
    fn l_l(l: &[i64], r: &[i64]) -> AST<'ast> {
        AST::Number(1)
    }
}

impl<'ast> Iterator for Eq<'ast> {
    type Item = AST<'ast>;
    fn next(&mut self) -> Option<Self::Item> {
        match (self.lvalue, self.rvalue) {
            (&AST::Number(l), &AST::Number(r)) => Some(Self::a_a(l, r)),
            _ => None,
        }
    }
}

impl<'a, 'ast> Iterator for &'a Eq<'ast> {
    type Item = AST<'ast>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
