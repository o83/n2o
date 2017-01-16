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
        AST::Number(l + r)
    }
    fn l_a(l: &'ast AST<'ast>, r: &'ast AST<'ast>) -> AST<'ast> {
        AST::Number(1)
    }
    fn a_l(l: &'ast AST<'ast>, r: &'ast AST<'ast>) -> AST<'ast> {
        AST::Number(1)
    }
    #[target_feature = "+avx"]
    fn v_v(l: &[i64], r: &[i64]) -> AST<'ast> {
        let a:Vec<i64> = l.iter().zip(r)
            .map(|(l,r)| l+r)
            .collect();
        AST::VecInt(a)
    }
    #[target_feature = "+avx"]
    fn v_a(l: &[i64], r: i64) -> AST<'ast> {
        let a:Vec<i64> = l.iter()
            .map(|x| x+r)
            .collect();
        AST::VecInt(a)
    }
}

impl<'ast> Iterator for Plus<'ast> {
    type Item = AST<'ast>;
    fn next(&mut self) -> Option<Self::Item> {
        match (self.lvalue, self.rvalue) {
            (&AST::Number(l), &AST::Number(r)) => Some(Self::a_a(l, r)),
            (&AST::VecInt(ref l), &AST::VecInt(ref r)) => Some(Self::v_v(l, r)),
            (&AST::Number(l), &AST::VecInt(ref r)) => Some(Self::v_a(r, l)),
            (&AST::VecInt(ref r), &AST::Number(l)) => Some(Self::v_a(r, l)),
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
