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

#[derive(Debug, Clone)]
pub struct Entry<'ast>(u16, AST<'ast>);

#[derive(Debug)]
pub struct Environment<'ast> {
    pub index: u64,
    pub parent: Option<&'ast Environment<'ast>>,
    pub values: Stack<Entry<'ast>>,
}

impl<'ast> Environment<'ast> {
    pub fn new_root() -> Result<Environment<'ast>, Error<'ast>> {
        Ok(Environment {
            parent: None,
            index: 0,
            values: Stack::with_capacity((!0 as u16) as usize),
        })
    }

    pub fn split(&'ast mut self) -> (&'ast mut Environment<'ast>, &'ast mut Environment<'ast>) {
        let f: *mut Environment<'ast> = self;
        let uf: &mut Environment<'ast> = unsafe { &mut *f };
        let us: &mut Environment<'ast> = unsafe { &mut *f };
        (uf, us)
    }

    pub fn new_child(&'ast mut self) -> Result<usize, Error> {
        match self.values.push_frame() {
            Ok(id) => Ok(id),
            Err(_) => Err(Error::InternalError),
        }
    }

    pub fn define(&'ast mut self, key: u16, value: AST<'ast>) -> Result<(), Error> {
        println!("Set {:?}:{:?} in Level {:?}", key, value, self.index);
        self.values.insert(Entry(key, value));
        Ok(())
    }

    pub fn get(&'ast self, key: u16, from: Option<usize>) -> Option<&'ast AST> {
        match self.values.get(|x| (*x).0 == key, from) {
            Some(x) => Some(&x.1),
            None => None,
        }
    }
}
