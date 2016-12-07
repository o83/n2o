#![feature(test)]
extern crate test;
extern crate kernel;

use kernel::commands::*;
use kernel::commands::ast::*;
use kernel::streams::interpreter::Interpreter;

#[test]
pub fn k_ariph() {
    assert_eq!(command::parse_Mex("1+2"),
               Ok(AST::Verb(Verb::Plus, AST::Number(1).boxed(), AST::Number(2).boxed())));

    assert_eq!(command::parse_Mex("1+2*4"),
               Ok(AST::Verb(Verb::Plus,
                            AST::Number(1).boxed(),
                            AST::Verb(Verb::Times,
                                      Box::new(AST::Number(2)),
                                      Box::new(AST::Number(4)))
                                .boxed())));
}

#[test]
pub fn k_list() {
    assert_eq!(command::parse_Mex("(1;2;3;4)"),
               Ok(AST::List(AST::Cons(AST::Number(1).boxed(),
                                      AST::Cons(AST::Number(2).boxed(),
                                                AST::Cons(AST::Number(3).boxed(),
                                                          AST::Number(4).boxed())
                                                    .boxed())
                                          .boxed())
                   .boxed())));
}

#[test]
pub fn k_symbols() {
    assert_eq!(command::parse_Mex("`a`b`c;`1`1`1"),
    Ok(AST::Cons(AST::Call(AST::Symbol(String::from("a")).boxed(), AST::Call(AST::Symbol(String::from("b")).boxed(), AST::Symbol(String::from("c")).boxed()).boxed()).boxed(),
    AST::Call(AST::Symbol(String::from("")).boxed(),
    AST::Call(AST::Number(1).boxed(),
    AST::Call(AST::Symbol(String::from("")).boxed(),
    AST::Call(AST::Number(1).boxed(), AST::Call(AST::Symbol(String::from("")).boxed(), AST::Number(1).boxed()).boxed()).boxed()).boxed()).boxed())
    .boxed())));
}

#[test]
pub fn k_assign() {
    assert_eq!(command::parse_Mex("a:b:c:1"),
               Ok(AST::Assign(AST::Name(String::from("a")).boxed(),
                              AST::Assign(AST::Name(String::from("b")).boxed(),
                                          AST::Assign(AST::Name(String::from("c")).boxed(),
                                                      AST::Number(1).boxed())
                                              .boxed())
                                  .boxed())));
}

#[test]
pub fn k_plus() {
    let mut i = Interpreter::new().unwrap();
    assert_eq!(format!("{}",
                       i.run(command::parse_Mex("2+5").unwrap())
                           .unwrap()),
               "7");
}

#[test]
pub fn k_func() {
    assert_eq!(format!("{:?}", command::parse_Mex("{x*2}[(1;2;3)]")),
               "Ok(Call(Lambda(Name(\"x\"), Verb(Times, Name(\"x\"), Number(2))), \
                List(Cons(Number(1), Cons(Number(2), Number(3))))))");
}

#[test]
pub fn k_adverb() {
    assert_eq!(format!("{:?}", command::parse_Mex("{x+2}/(1;2;3)")),
               "Ok(Adverb(Over, Lambda(Name(\"x\"), Verb(Plus, Name(\"x\"), Number(2))), \
                List(Cons(Number(1), Cons(Number(2), Number(3))))))");
}


#[test]
pub fn k_reduce() {
    assert_eq!(format!("{:?}",
                       command::parse_Mex("+/{x*y}[(1;3;4;5;6);(2;6;2;1;3)]")),
               "Ok(Adverb(Over, Verb(Plus, Nil, Nil), Call(Lambda(Name(\"x\"), Verb(Times, \
                Name(\"x\"), Name(\"y\"))), Dict(Cons(List(Cons(Number(1), Cons(Number(3), \
                Cons(Number(4), Cons(Number(5), Number(6)))))), List(Cons(Number(2), \
                Cons(Number(6), Cons(Number(2), Cons(Number(1), Number(3)))))))))))");
}

#[test]
pub fn k_repl() {
    let mut i = Interpreter::new().unwrap();
    assert_eq!(format!("{}",
                       i.run(command::parse_Mex("y:3;add:{[x]y};f:{[y]add y};f 1").unwrap())
                           .unwrap()),
               "3");
}

#[test]
pub fn k_repl1() {
    let mut i = Interpreter::new().unwrap();
    assert_eq!(format!("{}",
                       i.run(command::parse_Mex("y:3;addy:{[x]y};f:{[g;y]g y};f[addy;1]").unwrap())
                           .unwrap()),
               "3");
}

#[test]
pub fn k_repl2() {
    let mut i = Interpreter::new().unwrap();
    assert_eq!(format!("{}",
                       i.run(command::parse_Mex("xo:{1};z:{[x]xo x};d:{[x]z x};e:{[x]d x};e[3]").unwrap())
                           .unwrap()),
               "1");
}

#[test]
pub fn k_factorial() {
    let mut i = Interpreter::new().unwrap();
    assert_eq!(format!("{}",
                       i.run(command::parse_Mex("fac:{$[x=0;1;x*fac[x-1]]};fac 20").unwrap())
                           .unwrap()),
               "2432902008176640000");
}

#[test]
pub fn k_cond() {
    let mut i = Interpreter::new().unwrap();
    assert_eq!(format!("{}",
                       i.run(command::parse_Mex("a:{[x;y]$[x y;20;10]};a[{x};10]").unwrap())
                           .unwrap()),
               "20");
}

#[test]
pub fn k_cond2() {
    let mut i = Interpreter::new().unwrap();
    assert_eq!(format!("{}",
                       i.run(command::parse_Mex("a:{[x;y]$[x y;20;10]};a[{x};0]").unwrap())
                           .unwrap()),
               "10");

}
