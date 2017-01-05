use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};

pub struct Handle<T>(UnsafeCell<T>);

impl<T> Handle<T> {
    pub fn borrow(&self) -> &T {
        unsafe { &*self.0.get() }
    }

    pub fn borrow_mut(&self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }
}

pub fn new<T>(t: T) -> Handle<T> {
    Handle(UnsafeCell::new(t))
}

impl<T> Deref for Handle<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.borrow()
    }
}

impl<T> DerefMut for Handle<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.borrow_mut()
    }
}

pub fn split<T>(t: &mut T) -> (&mut T, &mut T) {
    let f: *mut T = t;
    let uf: &mut T = unsafe { &mut *f };
    let us: &mut T = unsafe { &mut *f };
    (uf, us)
}