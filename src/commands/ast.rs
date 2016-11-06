
#[derive(Debug)]
pub enum AST {
    Integer(u64),
    Symbol(String),
    Float(f64),
}
