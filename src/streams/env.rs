use std::fmt;
use std::hash::BuildHasherDefault;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::iter;
use std::vec;
use commands::ast::*;
use fnv::*;
use streams::stack::Stack;
use std::cell::UnsafeCell;

#[derive(Debug, Clone)]
pub struct Entry<'ast>(pub u16, pub &'ast AST<'ast>);

#[derive(Debug)]
pub struct Environment<'ast> {
    stack: UnsafeCell<Stack<Entry<'ast>>>,
}

impl<'ast> Environment<'ast> {
    pub fn new_root() -> Result<Environment<'ast>, Error> {
        Ok(Environment { stack: UnsafeCell::new(Stack::with_capacity(10000 as usize)) })
    }

    pub fn new_child(&'ast self) -> Result<usize, Error> {
        let stack = unsafe { &mut *self.stack.get() };
        match stack.push_frame() {
            Ok(id) => Ok(id),
            Err(_) => Err(Error::InternalError),
        }
    }

    pub fn define(&'ast self, key: u16, value: &'ast AST<'ast>) -> Result<(), Error> {
        let stack = unsafe { &mut *self.stack.get() };
        stack.insert(Entry(key, value));
        Ok(())
    }

    pub fn define_batch(&'ast self, items: &'ast [Entry]) -> Result<(), Error> {
        let stack = unsafe { &mut *self.stack.get() };
        stack.insert_many(items);
        Ok(())
    }

    pub fn get(&'ast self, key: u16, from: Option<usize>) -> Option<&'ast AST> {
        let stack = unsafe { &mut *self.stack.get() };
        match stack.get(|x| (*x).0 == key, from) {
            Some(x) => Some(&x.1),
            None => None,
        }
    }

    pub fn clean(&self) {
        let stack = unsafe { &mut *self.stack.get() };
        stack.clean();
    }
}
