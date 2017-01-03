use commands::ast::AST;

#[derive(Debug)]
pub enum Poll<T> {
    Yield(T),
    End,
}

#[derive(Debug)]
pub enum Error {
    RuntimeError,
}

#[derive(Debug)]
pub enum Context<'a> {
    Cont(usize),
    Node(&'a AST<'a>),
    Nil,
}

pub trait Task<'a> {
    fn init(&'a mut self);
    fn poll(&mut self, c: Context<'a>) -> Result<Poll<Context<'a>>, Error>;
}