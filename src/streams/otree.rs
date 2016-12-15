use std::{mem, ptr, isize};
use commands::ast::*;

#[derive(Debug, PartialEq)]
pub enum Error {
    Capacity,
    InvalidOperation,
}

pub type Key = u16;
pub type Value<'a> = AST<'a>;

#[derive(Debug,PartialEq,Clone)]
pub struct Entry<'a>(Key, Value<'a>);

#[derive(Debug,Clone)]
pub struct Node<'a> {
    bounds: (usize, usize),
    parent: Option<&'a Node<'a>>,
}

#[derive(Debug)]
pub struct Tree<'a> {
    nodes: Vec<Node<'a>>,
    items: Vec<Entry<'a>>,
}

impl<'a> Tree<'a> {
    pub fn with_capacity(cap: usize) -> Self {
        Tree {
            nodes: Vec::with_capacity(cap),
            items: Vec::with_capacity(cap),
        }
    }

    pub fn last_node(&'a mut self) -> Option<&'a mut Node<'a>> {
        self.nodes.last_mut()
    }

    pub fn new_node(&'a mut self, n: Node<'a>) {
        self.nodes.push(n);
    }

    pub fn insert(&mut self, item: Entry<'a>) {
        self.items.push(item);
    }

    pub fn insert_many(&mut self, items: &[Entry]) {}

    pub fn get(&'a self, n: &'a Node<'a>, key: Key) -> Option<(Key, &Node<'a>)> {
        for i in n.bounds.0..n.bounds.1 + 1 {
            if self.items[i].0 == key {
                return Some((self.items[i].0, n));
            }
        }
        match n.parent {
            None => None,
            Some(p) => self.get(p, key),
        }
    }

    pub fn clean(&mut self) {
        unsafe {
            self.items.set_len(0);
            self.nodes.set_len(0);
        }
    }
}