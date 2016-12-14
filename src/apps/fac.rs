extern crate kernel;

use kernel::commands::*;
use kernel::commands::ast::*;
use kernel::streams::interpreter::*;

fn main() {
    let mut i = Interpreter::new().unwrap();
    // let eval = &"f:{[x]x};{[x]a:f[x];a+a}1".to_string();
    let eval = &"f:{[x]$[0=x;1;x*f[x-1]]};fac[5]".to_string();
    let code = i.parse(eval);
    let r = i.run(code).unwrap();
    println!("Result: {:?}", r);
    i.gc();
}