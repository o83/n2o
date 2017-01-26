use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use std::fmt;
use core::marker::PhantomData;
use std::mem;


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


pub struct UnsafeShared<T: ?Sized> {
    pointer: *const T,
    _marker: PhantomData<T>,
}


impl<T: ?Sized> UnsafeShared<T> {
    pub unsafe fn new(ptr: *mut T) -> Self {
        UnsafeShared {
            pointer: ptr,
            _marker: PhantomData,
        }
    }
}

unsafe impl<T: Send + ?Sized> Send for UnsafeShared<T> {}

impl<T: ?Sized> Clone for UnsafeShared<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for UnsafeShared<T> {}

impl<T: ?Sized> Deref for UnsafeShared<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        unsafe { mem::transmute(&*self.pointer) }
    }
}


#[inline]
pub fn split<T>(t: &mut T) -> (&mut T, &mut T) {
    let f: *mut T = t;
    let uf: &mut T = unsafe { &mut *f };
    let us: &mut T = unsafe { &mut *f };
    (uf, us)
}

#[inline]
pub fn into_raw<T>(t: &mut T) -> *mut T {
    t as *mut T
}

#[inline]
pub fn from_raw<'a, T: 'a>(t: *mut T) -> &'a mut T {
    unsafe { &mut *t }
}

#[inline]
pub fn with<'a, T: 'a, F: 'a, R: 'a>(t: &mut T, mut f: F) -> R
    where F: FnMut(&'a mut T) -> R + 'a
{
    f(unsafe { &mut *(t as *mut T) })
}

#[inline]
pub fn with_raw<T, F, R>(t: &mut T, mut f: F) -> R
    where F: FnMut(*mut T) -> R
{
    f(t as *mut T)
}
