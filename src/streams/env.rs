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
pub struct Entry<'a>(pub u16, pub &'a AST<'a>);

#[derive(Debug)]
pub struct Environment<'a> {
    stack: UnsafeCell<Stack<Entry<'a>>>,
}

impl<'a> Environment<'a> {
    pub fn new_root() -> Result<Environment<'a>, Error> {
        Ok(Environment { stack: UnsafeCell::new(Stack::with_capacity(10000 as usize)) })
    }

    pub fn new_child(&'a self) -> Option<usize> {
        let stack = unsafe { &mut *self.stack.get() };
        match stack.push_frame() {
            Ok(id) => Some(id),
            Err(_) => None,
        }
    }

    pub fn define(&'a self, key: u16, value: &'a AST<'a>) -> Result<(), Error> {
        println!("DEFINE key: {:?}, value: {:?}", key, value);
        let stack = unsafe { &mut *self.stack.get() };
        stack.insert(Entry(key, value));
        Ok(())
    }

    pub fn define_batch(&'a self, items: &'a [Entry]) -> Result<(), Error> {
        let stack = unsafe { &mut *self.stack.get() };
        stack.insert_many(items);
        Ok(())
    }

    pub fn get(&'a self, key: u16, from: Option<usize>) -> Option<&'a AST> {
        let stack = unsafe { &mut *self.stack.get() };
        println!("SEARCH KEY: {:?}, STACK: {:?}", key, &stack);
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
