use std::fmt;
use std::fmt::Debug;

#[derive(PartialEq, Clone, Debug)]
struct Node {
    bounds: (usize, usize),
    parent: Option<usize>,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.parent {
            Some(parent) => write!(f, "[{:?}—{:?}]", self.bounds, parent),
            _ => write!(f, "[{:?}—(root)]", self.bounds),
        }
    }
}

// Node index inside otree store.
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct NodeId(usize);

#[derive(Debug)]
pub struct Tree<T> {
    nodes: Vec<Node>,
    items: Vec<T>,
}

// TODO: Remove T: Debug
impl<T: Debug> Tree<T> {
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

    pub fn len(&self) -> (usize, usize) {
        (self.nodes.len(), self.items.len())
    }

    pub fn dump(&self) {
        for i in self.items[0..self.items.len()].iter() {
            println!("item {:?}", i);
        }
        for i in self.nodes[0..self.nodes.len()].iter() {
            println!("node {}", i);
        }
    }

    #[inline]
    pub fn last(&self) -> NodeId {
        NodeId(self.nodes.len() - 1)
    }

    pub fn append_node(&mut self, n: NodeId) -> NodeId {
        let bound = self.nodes.last().expect("There is no root in otree.").bounds.1;
        self.nodes.push(Node {
            bounds: (bound, bound),
            parent: Some(n.0),
        });
        NodeId(self.nodes.len() - 1)
    }

    pub fn insert(&mut self, item: T) {
        self.items.push(item);
        let mut n = self.nodes.last_mut().unwrap();
        n.bounds.1 += 1;
    }

    pub fn get<'a, F>(&'a self, n: NodeId, mut f: F) -> Option<(&'a T, NodeId)>
        where for<'r> F: FnMut(&'r &T) -> bool
    {
        let nd = self.nodes.get(n.0).expect("Error getting node.");
        for i in self.items[nd.bounds.0..nd.bounds.1].iter().rev() {
            if f(&i) {
                return Some((i, n));
            }
        }
        match nd.parent {
            Some(p) => self.get(NodeId(p), f),
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