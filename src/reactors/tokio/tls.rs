
use std::cell::Cell;
use std::marker;
use std::thread::LocalKey;

#[macro_export]
macro_rules! scoped_thread_local {
    (static $name:ident: $ty:ty) => (
        static $name: ::reactors::tokio::tls::ScopedKey<$ty> = ::reactors::tokio::tls::ScopedKey {
            inner: {
                thread_local!(static FOO: ::std::cell::Cell<usize> = {
                    ::std::cell::Cell::new(0)
                });
                &FOO
            },
            _marker: ::std::marker::PhantomData,
        };
    )
}

pub struct ScopedKey<T> {
    #[doc(hidden)]
    pub inner: &'static LocalKey<Cell<usize>>,
    #[doc(hidden)]
    pub _marker: marker::PhantomData<T>,
}

unsafe impl<T> Sync for ScopedKey<T> {}

impl<T> ScopedKey<T> {
    pub fn set<F, R>(&'static self, t: &T, f: F) -> R
        where F: FnOnce() -> R
    {
        struct Reset {
            key: &'static LocalKey<Cell<usize>>,
            val: usize,
        }
        impl Drop for Reset {
            fn drop(&mut self) {
                self.key.with(|c| c.set(self.val));
            }
        }
        let prev = self.inner.with(|c| {
            let prev = c.get();
            c.set(t as *const T as usize);
            prev
        });
        let _reset = Reset {
            key: self.inner,
            val: prev,
        };
        f()
    }

    pub fn with<F, R>(&'static self, f: F) -> R
        where F: FnOnce(&T) -> R
    {
        let val = self.inner.with(|c| c.get());
        assert!(val != 0,
                "cannot access a scoped thread local variable without calling `set` first");
        unsafe { f(&*(val as *const T)) }
    }

    pub fn is_set(&'static self) -> bool {
        self.inner.with(|c| c.get() != 0)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::sync::mpsc::{channel, Sender};
    use std::thread;

    scoped_thread_local!(static FOO: u32);

    #[test]
    fn smoke() {
        scoped_thread_local!(static BAR: u32);

        assert!(!BAR.is_set());
        BAR.set(&1, || {
            assert!(BAR.is_set());
            BAR.with(|slot| {
                assert_eq!(*slot, 1);
            });
        });
        assert!(!BAR.is_set());
    }

    #[test]
    fn cell_allowed() {
        scoped_thread_local!(static BAR: Cell<u32>);

        BAR.set(&Cell::new(1), || {
            BAR.with(|slot| {
                assert_eq!(slot.get(), 1);
            });
        });
    }

    #[test]
    fn scope_item_allowed() {
        assert!(!FOO.is_set());
        FOO.set(&1, || {
            assert!(FOO.is_set());
            FOO.with(|slot| {
                assert_eq!(*slot, 1);
            });
        });
        assert!(!FOO.is_set());
    }

    #[test]
    fn panic_resets() {
        struct Check(Sender<u32>);
        impl Drop for Check {
            fn drop(&mut self) {
                FOO.with(|r| {
                    self.0.send(*r).unwrap();
                })
            }
        }

        let (tx, rx) = channel();
        let t = thread::spawn(|| {
            FOO.set(&1, || {
                let _r = Check(tx);

                FOO.set(&2, || panic!());
            });
        });

        assert_eq!(rx.recv().unwrap(), 1);
        assert!(t.join().is_err());
    }
}
