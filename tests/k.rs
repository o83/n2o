#![feature(test)]
extern crate test;
extern crate kernel;

use kernel::commands::*;
use kernel::commands::ast::*;
use kernel::streams::interpreter::Interpreter;

#[test]
pub fn k_ariph() {
    let ref mut x = ast::parse(&"1+2".to_string());
    assert_eq!(*x,
               AST::Verb(Verb::Plus, AST::Number(1).boxed(), AST::Number(2).boxed()));

    let ref mut y = ast::parse(&"1+2*4".to_string());
    assert_eq!(*y,
               AST::Verb(Verb::Plus,
                         AST::Number(1).boxed(),
                         AST::Verb(Verb::Times,
                                   Box::new(AST::Number(2)),
                                   Box::new(AST::Number(4)))
                             .boxed()));
}

#[test]
pub fn k_list() {
    let ref mut x = ast::parse(&"(1;2;3;4)".to_string());
    assert_eq!(*x,
               AST::List(AST::Cons(AST::Number(1).boxed(),
                                   AST::Cons(AST::Number(2).boxed(),
                                             AST::Cons(AST::Number(3).boxed(),
                                                       AST::Number(4).boxed())
                                                 .boxed())
                                       .boxed())
                   .boxed()));
}

#[test]
pub fn k_symbols() {
    let ref mut x = ast::parse(&"`a`b`c;`1`1`1".to_string());
    assert_eq!(*x,
    AST::Cons(AST::Call(AST::Symbol(String::from("a")).boxed(), AST::Call(AST::Symbol(String::from("b")).boxed(), AST::Symbol(String::from("c")).boxed()).boxed()).boxed(),
    AST::Call(AST::Symbol(String::from("")).boxed(),
    AST::Call(AST::Number(1).boxed(),
    AST::Call(AST::Symbol(String::from("")).boxed(),
    AST::Call(AST::Number(1).boxed(), AST::Call(AST::Symbol(String::from("")).boxed(), AST::Number(1).boxed()).boxed()).boxed()).boxed()).boxed())
    .boxed()));
}

#[test]
pub fn k_assign() {
    let ref mut x = ast::parse(&"a:b:c:1".to_string());
    assert_eq!(*x,
               AST::Assign(AST::NameInt(0).boxed(),
                           AST::Assign(AST::NameInt(1).boxed(),
                                       AST::Assign(AST::NameInt(2).boxed(),
                                                   AST::Number(1).boxed())
                                           .boxed())
                               .boxed()));
}

#[test]
pub fn k_plus() {
    let mut i = Interpreter::new().unwrap();
    let ref mut x = ast::parse(&"2+5".to_string());
    assert_eq!(format!("{}", i.run(x).unwrap()), "7");
}

#[test]
pub fn k_func() {
    let ref mut x = ast::parse(&"{x*2}[(1;2;3)]".to_string());
    assert_eq!(format!("{:?}", x),
               "Call(Lambda(NameInt(0), Verb(Times, NameInt(0), Number(2))), \
                List(Cons(Number(1), Cons(Number(2), Number(3)))))");
}

#[test]
pub fn k_adverb() {
    let ref mut x = ast::parse(&"{x+2}/(1;2;3)".to_string());
    assert_eq!(format!("{:?}", x),
               "Adverb(Over, Lambda(NameInt(0), Verb(Plus, NameInt(0), Number(2))), \
                List(Cons(Number(1), Cons(Number(2), Number(3)))))");
}


#[test]
pub fn k_reduce() {
    let ref mut x = ast::parse(&"+/{x*y}[(1;3;4;5;6);(2;6;2;1;3)]".to_string());
    assert_eq!(format!("{:?}", x),
               "Adverb(Over, Verb(Plus, Nil, Nil), Call(Lambda(NameInt(0), Verb(Times, \
                NameInt(0), NameInt(1))), Dict(Cons(List(Cons(Number(1), Cons(Number(3), \
                Cons(Number(4), Cons(Number(5), Number(6)))))), List(Cons(Number(2), \
                Cons(Number(6), Cons(Number(2), Cons(Number(1), Number(3))))))))))");
}

#[test]
pub fn k_repl() {
    let mut i = Interpreter::new().unwrap();
    let ref mut x = ast::parse(&"y:3;add:{[x]y};f:{[y]add y};f 1".to_string());
    assert_eq!(format!("{}", i.run(x).unwrap()), "3");
}

#[test]
pub fn k_repl1() {
    let mut i = Interpreter::new().unwrap();
    let ref mut x = ast::parse(&"y:3;addy:{[x]y};f:{[g;y]g y};f[addy;1]".to_string());
    assert_eq!(format!("{}", i.run(x).unwrap()), "3");
}

#[test]
pub fn k_repl2() {
    let mut i = Interpreter::new().unwrap();
    let ref mut x = ast::parse(&"xo:{1};z:{[x]xo x};d:{[x]z x};e:{[x]d x};e[3]".to_string());
    assert_eq!(format!("{}", i.run(x).unwrap()), "1");
}

#[test]
pub fn k_factorial() {
    let mut i = Interpreter::new().unwrap();
    let ref mut x = ast::parse(&"fac:{$[x=0;1;x*fac[x-1]]};fac 20".to_string());
    assert_eq!(format!("{}", i.run(x).unwrap()), "2432902008176640000");
}

#[test]
pub fn k_cond() {
    let mut i = Interpreter::new().unwrap();
    let ref mut x = ast::parse(&"a:{[x;y]$[x y;20;10]};a[{x};10]".to_string());
    assert_eq!(format!("{}", i.run(x).unwrap()), "20");
}

#[test]
pub fn k_cond2() {
    let mut i = Interpreter::new().unwrap();
    let ref mut x = ast::parse(&"a:{[x;y]$[x y;20;10]};a[{x};0]".to_string());
    assert_eq!(format!("{}", i.run(x).unwrap()), "10");

}
