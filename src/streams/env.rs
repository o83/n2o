use std::fmt;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::iter;
use std::vec;
use commands::ast::*;

#[derive(PartialEq)]
pub struct Environment {
    parent: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, AST>,
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.parent {
            Some(ref parent) => write!(f, "{:?}", self.values),
            None => write!(f, "{:?} ", self.values),
        }
    }
}

impl fmt::Debug for Environment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.parent {
            Some(ref parent) => write!(f, "{:?}", self.values),
            None => write!(f, "{:?} ", self.values),
        }
    }
}

impl Environment {
    pub fn new_root() -> Result<Rc<RefCell<Environment>>, Error> {
        let mut env = Environment {
            parent: None,
            values: HashMap::new(),
        };
        Ok(Rc::new(RefCell::new(env)))
    }

    pub fn new_child(parent: Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        let env = Environment {
            parent: Some(parent),
            values: HashMap::new(),
        };
        Rc::new(RefCell::new(env))
    }

    pub fn define(&mut self, key: String, value: AST) -> Result<(), Error> {
        self.values.insert(key, value);
        Ok(())
    }

    pub fn set(&mut self, key: String, value: AST) -> Result<(), Error> {
        if self.values.contains_key(&key) {
            self.values.insert(key, value);
            Ok(())
        } else {
            match self.parent {
                Some(ref parent) => parent.borrow_mut().set(key, value),
                None => {
                    Err(Error::EvalError {
                        desc: "Can't set! an undefined variable".to_string(),
                        ast: value,
                    })
                }
            }
        }
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

    pub fn get_root(env_ref: Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        let env = env_ref.borrow();
        match env.parent {
            Some(ref parent) => Environment::get_root(parent.clone()),
            None => env_ref.clone(),
        }
    }
}
