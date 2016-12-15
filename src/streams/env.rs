use std::fmt;
use std::hash::BuildHasherDefault;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::iter;
use std::vec;
use commands::ast::*;
use fnv::*;
use std::cell::UnsafeCell;
use streams::otree::*;

#[derive(Debug)]
pub struct Environment<'a> {
    tree: UnsafeCell<Tree<'a>>,
}

impl<'a> Environment<'a> {
    pub fn new_root() -> Result<Environment<'a>, Error> {
        let mut s = tree::with_capacity(10000 as usize);
        Ok(Environment { tree: UnsafeCell::new(s) })
    }

    pub fn new_child(&'a self) -> Option<usize> {
        let tree = unsafe { &mut *self.tree.get() };
        let ln = tree.last_node();
        let b = ln.bounds.1+1;
        let nn = Node {
            parent: Some(ln),
            bounds: (b,b),
        }
        tree.new_node(nn);
            }

    pub fn define(&'a self, key: u16, value: &'a AST<'a>) -> Result<(), Error> {
        let tree = unsafe { &mut *self.tree.get() };
        let frame = tree.last_frame_id();
        tree.insert(Entry(key, value, frame));
        println!("Env::Define {:?}:{:?}  Frame {:?}", key, value, &frame);
        println!("Env::tree {:?}", &tree);
        Ok(())
    }

    pub fn define_batch(&'a self, items: &'a [Entry]) -> Result<(), Error> {
        let tree = unsafe { &mut *self.tree.get() };
        tree.insert_many(items);
        Ok(())
    }

    pub fn get(&'a self, key: u16, from: Option<usize>) -> Option<(&'a AST, usize)> {
        let tree = unsafe { &mut *self.tree.get() };
        match tree.get(|x| (*x).0 == key, from) {
            Some(x) => Some((&x.1, x.2)),
            None => None,
        }
    }

    pub fn clean(&self) {
        let tree = unsafe { &mut *self.tree.get() };
        tree.clean();
    }
}
