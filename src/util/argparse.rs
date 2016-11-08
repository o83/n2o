// #
//
// argparse.rs
// Copyright (C) 2016 Lynx ltd. <anton@algotradinghub.com>
// Created by Anton Kundenko.
//

use std::env;
use std::collections::HashMap;

pub struct ArgParser<F> {
    args: Vec<String>,
    funcs: HashMap<String, F>,
}

impl<F> ArgParser<F>
    where F: FnMut(&str)
{
    pub fn new() -> Self {
        ArgParser {
            args: env::args().collect(),
            funcs: HashMap::new(),
        }
    }

    pub fn arg(&mut self, prm: String, func: F) -> &mut Self {
        self.funcs.insert(prm, func);
        self
    }

    pub fn parse(&mut self) {
        let cnt = &self.args.len() - 1;
        assert_eq!(0, &cnt % 2);

        for i in (1..cnt).step_by(2) {
            let func = self.funcs.get_mut(&self.args[i]);
            match func {
                Some(mut f) => (&mut f)(&self.args[i + 1]),
                None => {
                    error!("Option {:?} is unknown.", &self.args[i]);
                }
            }
        }
    }
}
