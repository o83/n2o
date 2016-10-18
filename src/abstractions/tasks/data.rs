use std::prelude::v1::*;
use std::any::TypeId;
use std::cell::RefCell;
use std::hash::{BuildHasherDefault, Hasher};
use std::collections::HashMap;

use abstractions::tasks::task;

#[macro_export]
macro_rules! task_local {
    (static $NAME:ident: $t:ty = $e:expr) => (
        static $NAME: $crate::task::LocalKey<$t> = {
            fn __init() -> $t { $e }
            fn __key() -> ::std::any::TypeId {
                struct __A;
                ::std::any::TypeId::of::<__A>()
            }
            $crate::task::LocalKey {
                __init: __init,
                __key: __key,
            }
        };
    )
}

pub type LocalMap = RefCell<HashMap<TypeId, Box<Opaque>, BuildHasherDefault<IdHasher>>>;

pub fn local_map() -> LocalMap {
    RefCell::new(HashMap::default())
}

pub trait Opaque: Send {}
impl<T: Send> Opaque for T {}

pub struct LocalKey<T> {
    // "private" fields which have to be public to get around macro hygiene, not
    // included in the stability story for this type. Can change at any time.
    #[doc(hidden)]
    pub __key: fn() -> TypeId,
    #[doc(hidden)]
    pub __init: fn() -> T,
}

pub struct IdHasher {
    id: u64,
}

impl Default for IdHasher {
    fn default() -> IdHasher {
        IdHasher { id: 0 }
    }
}

impl Hasher for IdHasher {
    fn write(&mut self, _bytes: &[u8]) {
        // TODO: need to do something sensible
        panic!("can only hash u64");
    }

    fn write_u64(&mut self, u: u64) {
        self.id = u;
    }

    fn finish(&self) -> u64 {
        self.id
    }
}

impl<T: Send + 'static> LocalKey<T> {
    pub fn with<F, R>(&'static self, f: F) -> R
        where F: FnOnce(&T) -> R
    {
        let key = (self.__key)();
        task::with(|_, data| {
            let raw_pointer = {
                let mut data = data.borrow_mut();
                let entry = data.entry(key).or_insert_with(|| Box::new((self.__init)()));
                &**entry as *const Opaque as *const T
            };
            unsafe { f(&*raw_pointer) }
        })
    }
}
