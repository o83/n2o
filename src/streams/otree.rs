use std::{mem, ptr, isize};
use std::fmt::Debug;

#[derive(Debug,Clone)]
pub struct Node<'a> {
    pub bounds: (usize, usize),
    pub parent: Option<&'a Node<'a>>,
}

#[derive(Debug)]
pub struct Tree<'a, T> {
    nodes: Vec<Node<'a>>,
    items: Vec<T>,
}

// TODO: Remove T: Debug
impl<'a, T: Debug> Tree<'a, T> {
    pub fn with_capacity(cap: usize) -> Self {
        let mut n = Vec::with_capacity(cap);
        n.push(Node {
            bounds: (0, 0),
            parent: None,
        });
        Tree {
            nodes: n,
            items: Vec::with_capacity(cap),
        }
    }

    #[inline]
    pub fn last_node(&'a self) -> &'a Node<'a> {
        self.nodes.last().unwrap()
    }

    #[inline]
    pub fn first_node(&'a self) -> &'a Node<'a> {
        self.nodes.get(0).unwrap()
    }

    #[inline]
    pub fn split(&'a mut self) -> (&'a mut Self, &'a mut Self) {
        let f: *mut Tree<'a, T> = self;
        let uf: &mut Tree<'a, T> = unsafe { &mut *f };
        let us: &mut Tree<'a, T> = unsafe { &mut *f };
        (uf, us)
    }

    pub fn append_node(&'a mut self, n: &'a Node<'a>) -> &'a Node<'a> {
        let (s1, s2) = self.split();
        let nl = s1.last_node();
        s2.nodes.push(Node {
            bounds: (nl.bounds.1, nl.bounds.1),
            parent: Some(n),
        });
        s1.last_node()
    }

    pub fn alloc_node(&'a mut self) -> &'a Node<'a> {
        let (s1, s2) = self.split();
        let n = s1.last_node();
        s2.nodes.push(Node {
            bounds: (n.bounds.1, n.bounds.1),
            parent: Some(n),
        });
        s1.last_node()
    }

    pub fn insert(&mut self, item: T) {
        self.items.push(item);
        let mut n = self.nodes.last_mut().unwrap();
        n.bounds.1 += 1;
    }

    pub fn get<F>(&'a self, n: &'a Node<'a>, mut f: F) -> Option<(&'a T, &'a Node<'a>)>
        where for<'r> F: FnMut(&'r &T) -> bool
    {
        for i in self.items[n.bounds.0..n.bounds.1].iter().rev() {
            if f(&i) {
                return Some((i, n));
            }
        }
        match n.parent {
            Some(p) => self.get(p, f),
            None => None,
        }
    }

    pub fn clean(&mut self) -> usize {
        let l = self.items.len();
        unsafe {
            self.items.set_len(0);
            self.nodes.set_len(0);
        }
        self.nodes.push(Node {
            bounds: (0, 0),
            parent: None,
        });
        l
    }
}