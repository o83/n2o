
// K primitives: http://kparc.com/lisp.txt

#[derive(Debug)]
pub enum AST {
    Integer(u64), Symbol(String),
    Float(f64),
    Append, Get, Set,
    Curry, Compose, Lambda(Box<AST>,Box<AST>),
    Expr, Nil,
    CommaList(Box<AST>), ColonList(Box<AST>),
    Cons(Box<AST>,Box<AST>), Car, Setq, Cond,
    Map, Reduce(Box<AST>), Min, Max,
    Plus(Box<AST>,Box<AST>), Minus(Box<AST>,Box<AST>), Mul(Box<AST>,Box<AST>), Div(Box<AST>,Box<AST>),
    Greater, Less, Equal,
    Length, Reverse, Member,
}
