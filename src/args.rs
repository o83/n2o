
// Args Parser by Anton

use std::{self, env};
use std::collections::HashMap;

pub struct Parser<'a> {
    args: Vec<String>,
    funcs: HashMap<&'a str, Box<FnMut(&str)>>,
}

impl<'a> Parser<'a> {
    pub fn new() -> Self {
        Parser {
            args: env::args().collect(),
            funcs: HashMap::new(),
        }
    }

    pub fn arg(&'a mut self, prm: &'a str, func: Box<FnMut(&str)>) -> &'a mut Self {
        self.funcs.insert(prm, func);
        self
    }

    pub fn parse(&'a mut self) {
        let cnt = &self.args.len() - 1;
        assert_eq!(0, &cnt % 2);

        for i in (1..cnt).step_by(2) {
            let func = self.funcs.get_mut(&self.args[i][..]);
            match func {
                Some(mut f) => (&mut f)(&self.args[i + 1]),
                None => {
                    error!("Option {:?} is unknown.", &self.args[i]);
                }
            }
        }
    }
}
