#![feature(test)]
extern crate test;
extern crate kernel;

use kernel::streams::otree::*;
use std::mem;

#[test]
pub fn tree() {
    let capacity = (!0 as u16) as usize;
    let mut tree: Tree = Tree::with_capacity(capacity);
}