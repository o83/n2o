use commands::ast::Value;
use commands::ast::{ASTNode, AST};

pub struct Plus<'ast> {
    lvalue: &'ast ASTNode<'ast>,
    rvalue: &'ast ASTNode<'ast>,
}

pub fn new<'ast>(lvalue: &'ast ASTNode<'ast>, rvalue: &'ast ASTNode<'ast>) -> Plus<'ast> {
    Plus {
        lvalue: lvalue,
        rvalue: rvalue,
    }
}

impl<'ast> Plus<'ast> {
    fn a_a(l: i64, r: i64) -> ASTNode<'ast> {
        ASTNode::AST(AST::Value(Value::Number(l + r)))
    }
    fn l_a(l: &'ast AST<'ast>, r: &'ast AST<'ast>) -> ASTNode<'ast> {
        ASTNode::AST(AST::Value(Value::Number(1)))
    }
    fn a_l(l: &'ast AST<'ast>, r: &'ast AST<'ast>) -> ASTNode<'ast> {
        ASTNode::AST(AST::Value(Value::Number(1)))
    }

    fn v_v(l: &[i64], r: &[i64]) -> ASTNode<'ast> {
        let a: Vec<i64> = l.iter()
            .zip(r)
            .map(|(l, r)| l + r)
            .collect();
        ASTNode::AST(AST::Value(Value::VecInt(a)))
    }

    fn v_a(l: &[i64], r: i64) -> ASTNode<'ast> {
        let a: Vec<i64> = l.iter()
            .map(|x| x + r)
            .collect();
        ASTNode::AST(AST::Value(Value::VecInt(a)))
    }
}

impl<'ast> Iterator for Plus<'ast> {
    type Item = ASTNode<'ast>;
    fn next(&mut self) -> Option<Self::Item> {
        match (self.lvalue, self.rvalue) {
            (&ASTNode::AST(AST::Value(Value::Number(l))), &ASTNode::AST(AST::Value(Value::Number(r)))) => {
                Some(Self::a_a(l, r))
            }
            (&ASTNode::AST(AST::Value(Value::VecInt(ref l))), &ASTNode::AST(AST::Value(Value::VecInt(ref r)))) => {
                Some(Self::v_v(l, r))
            }
            (&ASTNode::AST(AST::Value(Value::Number(l))), &ASTNode::AST(AST::Value(Value::VecInt(ref r)))) => {
                Some(Self::v_a(r, l))
            }
            (&ASTNode::AST(AST::Value(Value::VecInt(ref r))), &ASTNode::AST(AST::Value(Value::Number(l)))) => {
                Some(Self::v_a(r, l))
            }
            _ => None,
        }
    }
}

impl<'a, 'ast> Iterator for &'a Plus<'ast> {
    type Item = ASTNode<'ast>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
