pub mod plus;
pub mod minus;
pub mod eq;
pub mod mul;
pub mod div;

use commands::ast::*;

pub fn eval<'ast>(verb: Verb,
                  left: &'ast AST<'ast>,
                  right: &'ast AST<'ast>)
                  -> Result<AST<'ast>, Error<'ast>> {
    match verb {
        Verb::Plus => {
            let mut a = plus::new(left, right);
            Ok(a.next().unwrap())
        }
        // Verb::Minus => {
        //     let mut a = minus::new(left, right);
        //     Ok(a.next().unwrap())
        // }
        // Verb::Times => {
        //     let mut a = mul::new(left, right);
        //     Ok(a.next().unwrap())
        // }
        // Verb::Divide => {
        //     let mut a = div::new(left, right);
        //     Ok(a.next().unwrap())
        // }
        // Verb::Equal => {
        //     let mut a = eq::new(left, right);
        //     Ok(a.next().unwrap())
        // }
        x => {
            Err(Error::EvalError {
                desc: "Verb is not implemented".to_string(),
                ast: AST::Nil,
            })
        }
    }
}
