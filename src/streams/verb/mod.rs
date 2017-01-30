pub mod plus;
pub mod minus;
pub mod eq;
pub mod mul;
pub mod div;
pub mod dot;

use commands::ast::*;

pub fn eval<'ast>(verb: Verb, left: &'ast AST<'ast>, right: &'ast AST<'ast>) -> Result<AST<'ast>, Error> {
    match verb {
        Verb::Plus => {
            let mut a = plus::new(left, right);
            Ok(a.next().expect("Verb Plus"))
        }
        Verb::Minus => {
            let mut a = minus::new(left, right);
            Ok(a.next().expect("Verb Minus"))
        }
        Verb::Times => {
            let mut a = mul::new(left, right);
            Ok(a.next().expect("Verb Times"))
        }
        Verb::Divide => {
            let mut a = div::new(left, right);
            Ok(a.next().expect("Verb Divide"))
        }
        Verb::Equal => {
            let mut a = eq::new(left, right);
            Ok(a.next().expect("Verb Equal"))
        }
        x => {
            Err(Error::EvalError {
                desc: "Verb is not implemented".to_string(),
                ast: format!("{:?}", AST::Atom(Atom::Value(Value::Nil))),
            })
        }
    }
}