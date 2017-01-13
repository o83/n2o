
// Args Parser by Anton

use std::env;
use core::result::Result;

pub enum Error {
    ArgumentNotFound,
}

pub struct Parser {
    args: Vec<String>,
}

impl<'a> Parser {
    pub fn new() -> Self {
        Parser { args: env::args().collect() }
    }

    pub fn get(&mut self, arg: &str, hasval: bool) -> Result<Option<&String>, Error> {
        let mut it = self.args.iter();
        // omit an 0th argument (program name)
        let v = it.next();
        loop {
            if let Some(v) = it.next() {
                if v == arg {
                    return match hasval {
                        true => Ok(it.next()),
                        _ => Ok(None),
                    };
                }
            } else {
                return Err(Error::ArgumentNotFound);
            }
        }
    }
}
