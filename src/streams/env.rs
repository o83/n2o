use std::fmt;
use std::hash::BuildHasherDefault;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::iter;
use std::vec;
use commands::ast::*;
use fnv::*;
type Linked<K, V> = HashMap<K, V, BuildHasherDefault<FnvHasher>>;

#[derive(PartialEq, Debug, Clone)]
pub struct Environment<'ast> {
    pub index: u64,
    pub parent: Option<Rc<RefCell<Environment<'ast>>>>,
    pub values: Linked<u16, AST<'ast>>,
}

impl<'ast> Environment<'ast> {
    pub fn new_root() -> Result<Rc<RefCell<Environment<'ast>>>, Error<'ast>> {
        let mut env = Environment {
            parent: None,
            index: 0,
            values: FnvHashMap::with_capacity_and_hasher(10, Default::default()),
        };
        Ok(Rc::new(RefCell::new(env)))
    }

    pub fn new_child(parent: Rc<RefCell<Environment<'ast>>>) -> Rc<RefCell<Environment<'ast>>> {
        let idx = unsafe { (&*parent.as_unsafe_cell().get()).index };
        let env = Environment {
            parent: Some(parent),
            index: idx + 1,
            values: FnvHashMap::with_capacity_and_hasher(10, Default::default()),
        };
        Rc::new(RefCell::new(env))
    }

    pub fn index(parent: Rc<RefCell<Environment<'ast>>>) -> u64 {
        let a = unsafe { parent.as_unsafe_cell().get() };
        unsafe { (&*a).index }
    }

    pub fn define(&mut self, key: u16, value: AST<'ast>) -> Result<(), Error<'ast>> {
        self.values.insert(key, value);
        Ok(())
    }

    pub fn get(&self, key: &u16) -> Option<&'ast AST<'ast>> {
        match self.values.get(key) {
            Some(val) => Some(val),
            None => {
                match self.parent {
                    Some(ref parent) => parent.borrow().get(key),
                    None => None,
                }
            }
        }
    }

    pub fn find(&self, key: &u16) -> Option<(AST<'ast>, Rc<RefCell<Environment<'ast>>>)> {
        match self.values.get(key) {
            Some(val) => Some((val.clone(), Rc::new(RefCell::new(self.clone())))),
            None => {
                match self.parent {
                    Some(ref parent) => parent.borrow().find(key),
                    None => None,
                }
            }
        }
    }

    pub fn get_root(env_ref: Rc<RefCell<Environment<'ast>>>) -> Rc<RefCell<Environment<'ast>>> {
        let env = env_ref.borrow();
        match env.parent {
            Some(ref parent) => Environment::get_root(parent.clone()),
            None => env_ref.clone(),
        }
    }
}
