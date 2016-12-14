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
pub struct Entry<'a>(u16, &'a AST<'a>, usize);

#[derive(Debug)]
pub struct Environment<'a> {
    stack: UnsafeCell<Stack<Entry<'a>>>,
}

impl<'a> Environment<'a> {
    pub fn new_root() -> Result<Environment<'a>, Error> {
        let mut s = Stack::with_capacity(10000 as usize);
        s.push_frame();
        Ok(Environment { stack: UnsafeCell::new(s) })
    }

    pub fn new_child(&'a self) -> Option<usize> {
        let stack = unsafe { &mut *self.stack.get() };
        match stack.push_frame() {
            Ok(id) => Some(id),
            Err(_) => None,
        }
    }

    pub fn define(&'a self, key: u16, value: &'a AST<'a>) -> Result<(), Error> {
        let stack = unsafe { &mut *self.stack.get() };
        let frame = stack.last_frame_id();
        stack.insert(Entry(key, value, frame));
        println!("Env::Define {:?}:{:?}  Frame {:?}", key, value, &frame);
        println!("Env::Stack {:?}", &stack);
        Ok(())
    }

    pub fn define_batch(&'a self, items: &'a [Entry]) -> Result<(), Error> {
        let stack = unsafe { &mut *self.stack.get() };
        stack.insert_many(items);
        Ok(())
    }

    pub fn get(&'a self, key: u16, from: Option<usize>) -> Option<(&'a AST, usize)> {
        let stack = unsafe { &mut *self.stack.get() };
        match stack.get(|x| (*x).0 == key, from) {
            Some(x) => Some((&x.1, x.2)),
            None => None,
        }
    }

    pub fn clean(&self) {
        let stack = unsafe { &mut *self.stack.get() };
        stack.clean();
    }
}
