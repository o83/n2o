// Iterate over several streams

use core::ops::Index;

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
            pos: 0,
            next: 0,
        }
    }
}

pub struct FusionIntoIterator {
    inner: Fusion,
    // position markers just for example,
    // in real life here will be the matrix.
    pos: usize,
    next: usize,
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
        // just for showcase
        let (n, p) = (self.next, self.pos);
        self.next += 1;
        self.pos += 1;
        if self.next == 3 {
            self.next = 0;
        };
        if self.pos == self.inner.0.len() {
            return None;
        }
        let v: *const Return = unsafe { self[n].unpack().get(self.pos).unwrap() as *const Return };
        Some(v)
    }
}