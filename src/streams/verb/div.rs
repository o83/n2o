use commands::ast::Value;
use commands::ast::{ASTNode, AST};

pub struct Div<'ast> {
    lvalue: &'ast ASTNode<'ast>,
    rvalue: &'ast ASTNode<'ast>,
}

pub fn new<'ast>(lvalue: &'ast ASTNode<'ast>, rvalue: &'ast ASTNode<'ast>) -> Div<'ast> {
    Div {
        lvalue: lvalue,
        rvalue: rvalue,
    }
}

impl<'ast> Div<'ast> {
    fn a_a(l: i64, r: i64) -> ASTNode<'ast> {
        ASTNode::AST(AST::Value(Value::Number(l / r)))
    }
    fn l_a(l: &'ast AST<'ast>, r: &'ast AST<'ast>) -> ASTNode<'ast> {
        ASTNode::AST(AST::Value(Value::Number(1)))
    }
    fn a_l(l: &'ast AST<'ast>, r: &'ast AST<'ast>) -> ASTNode<'ast> {
        ASTNode::AST(AST::Value(Value::Number(1)))
    }
    fn l_l(l: &[i64], r: &[i64]) -> ASTNode<'ast> {
        let a: Vec<i64> = l.iter()
            .zip(r)
            .map(|(l, r)| l / r)
            .collect();
        ASTNode::AST(AST::Value(Value::VecInt(a)))
    }
}

impl<'ast> Iterator for Div<'ast> {
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

impl<'a, 'ast> Iterator for &'a Div<'ast> {
    type Item = ASTNode<'ast>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
