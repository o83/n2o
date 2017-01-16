use commands::ast::AST;

pub struct Minus<'ast> {
    lvalue: &'ast AST<'ast>,
    rvalue: &'ast AST<'ast>,
}

pub fn new<'ast>(lvalue: &'ast AST<'ast>, rvalue: &'ast AST<'ast>) -> Minus<'ast> {
    Minus {
        lvalue: lvalue,
        rvalue: rvalue,
    }
}

impl<'ast> Minus<'ast> {
    fn a_a(l: i64, r: i64) -> AST<'ast> {
        AST::Number(l - r)
    }
    fn l_a(l: &'ast AST<'ast>, r: &'ast AST<'ast>) -> AST<'ast> {
        AST::Number(1)
    }
    fn a_l(l: &'ast AST<'ast>, r: &'ast AST<'ast>) -> AST<'ast> {
        AST::Number(1)
    }
    #[target_feature = "+avx"]
    fn l_l(l: &[i64], r: &[i64]) -> AST<'ast> {
        AST::Number(1)
    }
}

impl<'ast> Iterator for Minus<'ast> {
    type Item = AST<'ast>;
    fn next(&mut self) -> Option<Self::Item> {
        match (self.lvalue, self.rvalue) {
            (&AST::Number(l), &AST::Number(r)) => Some(Self::a_a(l, r)),
            _ => None,
        }
    }
}

impl<'a, 'ast> Iterator for &'a Minus<'ast> {
    type Item = AST<'ast>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
