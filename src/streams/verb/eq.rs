use commands::ast::Value;
use commands::ast::{AST, ASTNode};

pub struct Eq<'ast> {
    lvalue: &'ast ASTNode<'ast>,
    rvalue: &'ast ASTNode<'ast>,
}

pub fn new<'ast>(lvalue: &'ast ASTNode<'ast>, rvalue: &'ast ASTNode<'ast>) -> Eq<'ast> {
    Eq {
        lvalue: lvalue,
        rvalue: rvalue,
    }
}

impl<'ast> Eq<'ast> {
    fn a_a(l: i64, r: i64) -> ASTNode<'ast> {
        ASTNode::AST(AST::Value(Value::Number(if r == l { 1 } else { 0 })))
    }
    fn l_a(l: &'ast AST<'ast>, r: &'ast AST<'ast>) -> ASTNode<'ast> {
        ASTNode::AST(AST::Value(Value::Number(1)))
    }
    fn a_l(l: &'ast AST<'ast>, r: &'ast AST<'ast>) -> ASTNode<'ast> {
        ASTNode::AST(AST::Value(Value::Number(1)))
    }
    fn l_l(l: &[i64], r: &[i64]) -> ASTNode<'ast> {
        ASTNode::AST(AST::Value(Value::Number(1)))
    }
}

impl<'ast> Iterator for Eq<'ast> {
    type Item = ASTNode<'ast>;
    fn next(&mut self) -> Option<Self::Item> {
        match (self.lvalue, self.rvalue) {
            (&ASTNode::AST(AST::Value(Value::Number(l))), &ASTNode::AST(AST::Value(Value::Number(r)))) => {
                Some(Self::a_a(l, r))
            }
            _ => None,
        }
    }
}

impl<'a, 'ast> Iterator for &'a Eq<'ast> {
    type Item = ASTNode<'ast>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
