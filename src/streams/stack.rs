use std::{mem, ptr, isize};

#[derive(Debug, PartialEq)]
pub enum Error {
    Capacity,
    InvalidOperation,
}
#[derive(Debug)]
pub struct Stack<T> {
    frames: Vec<usize>,
    items: Vec<T>,
}

impl<T: Clone> Stack<T> {
    // Use one variable for both: capacity and frames size
    // because we can't have more frames then stack capacity.
    pub fn with_capacity(cap: usize) -> Stack<T> {
        Stack {
            frames: Vec::with_capacity(cap),
            items: Vec::with_capacity(cap),
        }
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.items.capacity() == self.items.len()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.items.capacity()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    #[inline]
    pub fn free_len(&self) -> usize {
        self.capacity() - self.items.len()
    }

    #[inline]
    pub fn num_frames(&self) -> usize {
        self.frames.len()
    }

    #[inline]
    pub fn last_frame_id(&self) -> usize {
        self.frames.len() - 1
    }

    pub fn push_frame(&mut self) -> Result<usize, Error> {
        if self.is_full() {
            Err(Error::Capacity)
        } else {
            self.frames.push(self.items.len());
            Ok(self.last_frame_id())
        }
    }

    pub fn pop_frame(&mut self) -> Result<(), Error> {
        if self.items.is_empty() {
            Err(Error::InvalidOperation)
        } else {
            let frame_size = self.items.len() - *self.frames.last().unwrap();
            for i in 0..frame_size {
                self.items.pop();
            }
            self.frames.pop();
            Ok(())
        }
    }

    pub fn insert(&mut self, item: T) -> Result<(), Error> {
        if self.is_full() {
            Err(Error::Capacity)
        } else {
            self.items.push(item);
            Ok(())
        }
    }

    pub fn insert_many(&mut self, items: &[T]) -> Result<(), Error> {
        let ln_from = items.len();
        let ln_to = self.len();
        let cap = self.capacity();
        if (ln_from >= isize::MAX as usize) || (ln_from > self.free_len()) {
            return Err(Error::Capacity);
        }
        let to = self.items.as_mut_ptr();
        let from = items.as_ptr();
        unsafe {
            ptr::copy_nonoverlapping(from, to.offset(ln_to as isize), ln_from);
            let i = mem::replace(&mut self.items,
                                 Vec::from_raw_parts(to, ln_from + ln_to, cap));
            mem::forget(i);
        };
        Ok(())
    }

    pub fn insert_many_v2(&mut self, items: &[T]) -> Result<(), Error> {
        self.items.extend_from_slice(items);
        Ok(())
    }

    pub fn insert_many_v3(&mut self, items: &[T]) -> Result<(), Error> {
        self.items.extend(items.iter().cloned());
        Ok(())
    }

    // get(|item| (*item).key == 14, None)
    pub fn get<'a, F>(&'a self, f: F, from: Option<usize>) -> Option<&T>
        where for<'r> F: FnMut(&'r &T) -> bool
    {
        match from {
            Some(x) => self.items[..x + 1].iter().rev().find(f),
            None => self.items.iter().rev().find(f),
        }
    }

    pub fn clean(&mut self) {
        self.items.clear();
        self.frames.clear();
    }
}