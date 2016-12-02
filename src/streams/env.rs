use std::fmt;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::iter;
use std::vec;
use commands::ast::*;

#[derive(PartialEq, Clone)]
pub struct Environment {
    pub index: u64,
    pub parent: Option<Rc<RefCell<Environment>>>,
    pub values: HashMap<String, AST>,
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.parent {
            Some(ref parent) => write!(f, "LVL {:?} {:?}", self.index, self.values),
            None => write!(f, "LVL {:?} {:?}", self.index, self.values),
        }
    }
}

impl fmt::Debug for Environment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.parent {
            Some(ref parent) => write!(f, "LVL {:?} {:?}", self.index, self.values),
            None => write!(f, "LVL {:?} {:?}", self.index, self.values),
        }
    }
}

impl Environment {
    pub fn new_root() -> Result<Rc<RefCell<Environment>>, Error> {
        let mut env = Environment {
            parent: None,
            index: 0,
            values: HashMap::new(),
        };
        Ok(Rc::new(RefCell::new(env)))
    }

    pub fn new_child(parent: Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        let idx = unsafe { (&*parent.as_unsafe_cell().get()).index };
        let env = Environment {
            parent: Some(parent),
            index: idx + 1,
            values: HashMap::new(),
        };
        Rc::new(RefCell::new(env))
    }

    pub fn index(parent: Rc<RefCell<Environment>>) -> u64 {
        let a = unsafe { parent.as_unsafe_cell().get() };
        unsafe { (&*a).index }
    }

    pub fn define(&mut self, key: String, value: AST) -> Result<(), Error> {
        println!("Set {:?}:{:?} in LVL {:?}", key, value, self.index);
        self.values.insert(key, value);
        Ok(())
    }

    pub fn get(&self, key: &String) -> Option<AST> {
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

    pub fn find(&self, key: &String) -> Option<(AST, Rc<RefCell<Environment>>)> {
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
