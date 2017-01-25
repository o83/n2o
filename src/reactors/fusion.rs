// Iterate over several streams

use core::ops::Index;
use std::vec::IntoIter;
use std::iter::Cycle;
use core::iter::Filter;
use core::iter::FusedIterator;
use core::iter::Iterator;
use core::iter::Enumerate;
use core::iter::FlatMap;
use std::slice::Iter;
use std::iter::{self, Take, Repeat};

pub struct Quantifier<F>(FlatMap<Cycle<Enumerate<IntoIter<usize>>>, Take<Repeat<usize>>, F>)
    where F: FnMut((usize, usize)) -> Take<Repeat<usize>>;

impl<F> Quantifier<F>
    where F: FnMut((usize, usize)) -> Take<Repeat<usize>>
{
    pub fn new(v: Vec<usize>, f: F) -> Self {
        Quantifier(v.into_iter().enumerate().cycle().flat_map(f))
    }
}

impl<F> Iterator for Quantifier<F>
    where F: FnMut((usize, usize)) -> Take<Repeat<usize>>
{
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub struct Functor;

impl FnOnce<((usize, usize),)> for Functor {
    type Output = Take<Repeat<usize>>;
    extern "rust-call" fn call_once(self, args: ((usize, usize),)) -> Take<Repeat<usize>> {
        panic!("call_once()");
    }
}

impl FnMut<((usize, usize),)> for Functor {
    extern "rust-call" fn call_mut(&mut self, args: ((usize, usize),)) -> Take<Repeat<usize>> {
        iter::repeat((args.0).0).take((args.0).1)
    }
}

#[derive(Debug,Clone)]
pub enum Return {
    Io(u8),
    Bus(u16),
    Task(String),
    Unknown,
}

#[derive(Debug)]
pub enum Id {
    I(Vec<Return>),
    B(Vec<Return>),
    T(Vec<Return>),
}

// Here will be real IO, Channel, Task
pub struct Fusion(// IO
                  Id,
                  // BUS
                  Id,
                  // Tasks
                  Id);

impl Fusion {
    pub fn new() -> Self {
        Fusion(Id::I(vec![Return::Io(0u8);16]),
               Id::B(vec![Return::Bus(1u16);16]),
               Id::T(vec![Return::Task(String::from("a"));16]))
    }
}

impl IntoIterator for Fusion {
    type Item = *const Return;
    type IntoIter = FusionIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        FusionIntoIterator {
            inner: self,
            q: Quantifier::new(vec![1, 2, 3], Functor {}),
        }
    }
}

pub struct FusionIntoIterator {
    inner: Fusion,
    q: Quantifier<Functor>,
}

impl Id {
    pub fn len(&self) -> usize {
        match *self {
            Id::I(ref s) => s.len(),
            Id::B(ref s) => s.len(),
            Id::T(ref s) => s.len(),
        }
    }

    pub fn unpack(&self) -> &Vec<Return> {
        match *self {
            Id::I(ref s) => s,
            Id::B(ref s) => s,
            Id::T(ref s) => s,
        }
    }
}

impl Index<usize> for FusionIntoIterator {
    type Output = Id;

    fn index(&self, idx: usize) -> &Id {
        match idx {
            0 => &self.inner.0,
            1 => &self.inner.1,
            2 => &self.inner.2,            
            _ => panic!("Malformed id"),
        }
    }
}

impl Iterator for FusionIntoIterator {
    type Item = *const Return;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.q.next().unwrap();
        let v: *const Return = unsafe { self[id].unpack().get(0).unwrap() as *const Return };
        Some(v)

    }
}
