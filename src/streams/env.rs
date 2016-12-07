use std::fmt;
use std::hash::BuildHasherDefault;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::iter;
use std::vec;
use fnv::*;
use commands::ast::*;

type Linked<K, V> = HashMap<K, V, BuildHasherDefault<FnvHasher>>;

#[derive(PartialEq, Debug, Clone)]
pub struct Environment {
    pub index: u64,
    pub parent: Option<Rc<RefCell<Environment>>>,
    pub values: Linked<u16, AST>,
}

impl Environment {
    pub fn new_root() -> Result<Rc<RefCell<Environment>>, Error> {
        let mut env = Environment {
            parent: None,
            index: 0,
            values: FnvHashMap::with_capacity_and_hasher(10, Default::default())
        };
        Ok(Rc::new(RefCell::new(env)))
    }

    pub fn new_child(parent: Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        let idx = unsafe { (&*parent.as_unsafe_cell().get()).index };
        let env = Environment {
            parent: Some(parent),
            index: idx + 1,
            values: FnvHashMap::with_capacity_and_hasher(10, Default::default())
        };
        Rc::new(RefCell::new(env))
    }

    pub fn index(parent: Rc<RefCell<Environment>>) -> u64 {
        let a = unsafe { parent.as_unsafe_cell().get() };
        unsafe { (&*a).index }
    }

    pub fn define(&mut self, key: u16, value: AST) -> Result<(), Error> {
        self.values.insert(key, value);
        Ok(())
    }

    pub fn get(&self, key: &u16) -> Option<AST> {
        match self.values.get(key) {
            Some(val) => Some(val.clone()),
            None => {
                match self.parent {
                    Some(ref parent) => parent.borrow().get(key),
                    None => None,
                }
            }
        }
    }

    pub fn find(&self, key: &u16) -> Option<(AST, Rc<RefCell<Environment>>)> {
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

    pub fn get_root(env_ref: Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        let env = env_ref.borrow();
        match env.parent {
            Some(ref parent) => Environment::get_root(parent.clone()),
            None => env_ref.clone(),
        }
    }
}
