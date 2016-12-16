#![feature(test)]
extern crate test;
extern crate kernel;

use kernel::commands::*;
use kernel::commands::ast::*;
use kernel::streams::interpreter::Interpreter;

#[test]
pub fn k_ariph() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"1+2".to_string());
    assert_eq!(*code,
               AST::Verb(Verb::Plus, &AST::Number(1), &AST::Number(2)));

    let code = i.parse(&"1+2*4".to_string());
    assert_eq!(*code,
               AST::Verb(Verb::Plus,
                         &AST::Number(1),
                         &AST::Verb(Verb::Times, &AST::Number(2), &AST::Number(4))));
}

#[test]
pub fn k_list() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"(1;2;3;4)".to_string());
    assert_eq!(*code,
               AST::List(&AST::Cons(&AST::Number(1),
                                    &AST::Cons(&AST::Number(2),
                                               &AST::Cons(&AST::Number(3), &AST::Number(4))))));
}

#[test]
pub fn k_symbols() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"`a`b`c;`1`1`1".to_string());
    assert_eq!(*code,
               AST::Cons(&AST::Call(&AST::Symbol(String::from("a")),
                                    &AST::Call(&AST::Symbol(String::from("b")),
                                               &AST::Symbol(String::from("c")))),
                         &AST::Call(&AST::Symbol(String::from("")),
                                    &AST::Call(&AST::Number(1),
                                               &AST::Call(&AST::Symbol(String::from("")),
                                                          &AST::Call(&AST::Number(1),
                                                                     &AST::Call(&AST::Symbol(String::from("")),
                                                                                &AST::Number(1))))))));
}

#[test]
pub fn k_assign() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"a:b:c:1".to_string());
    assert_eq!(*code,
               AST::Assign(&AST::NameInt(0),
                           &AST::Assign(&AST::NameInt(1),
                                        &AST::Assign(&AST::NameInt(2), &AST::Number(1)))));
}

#[test]
pub fn k_plus() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"2+5".to_string());
    assert_eq!(format!("{}", i.run(code).unwrap()), "7");
}

#[test]
pub fn k_func() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"{x*2}[(1;2;3)]".to_string());
    assert_eq!(format!("{:?}", code),
               "Call(Lambda(NameInt(0), Verb(Times, NameInt(0), Number(2))), \
                List(Cons(Number(1), Cons(Number(2), Number(3)))))");
}

#[test]
pub fn k_adverb() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"{x+2}/(1;2;3)".to_string());
    assert_eq!(format!("{:?}", code),
               "Adverb(Over, Lambda(NameInt(0), Verb(Plus, NameInt(0), Number(2))), \
                List(Cons(Number(1), Cons(Number(2), Number(3)))))");
}


#[test]
pub fn k_reduce() {
    let mut i = Interpreter::new().unwrap();
    let ref mut code = i.parse(&"+/{x*y}[(1;3;4;5;6);(2;6;2;1;3)]".to_string());
    assert_eq!(format!("{:?}", code),
               "Adverb(Over, Verb(Plus, Nil, Nil), Call(Lambda(NameInt(0), Verb(Times, \
                NameInt(0), NameInt(1))), Dict(Cons(List(Cons(Number(1), Cons(Number(3), \
                Cons(Number(4), Cons(Number(5), Number(6)))))), List(Cons(Number(2), \
                Cons(Number(6), Cons(Number(2), Cons(Number(1), Number(3))))))))))");
}

#[test]
pub fn k_repl() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"y:3;add:{[x]y};f:{[x]add x};f 1".to_string());
    assert_eq!(format!("{}", i.run(code).unwrap()), "3");
}


#[test]
pub fn k_repl2() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"xo:{1};z:{[x]xo x};d:{[x]z x};e:{[x]d x};e[3]".to_string());
    assert_eq!(format!("{}", i.run(code).unwrap()), "1");
}

#[test]
pub fn k_factorial() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"fac:{$[x=0;1;x*fac[x-1]]};fac 20".to_string());
    assert_eq!(format!("{}", i.run(code).unwrap()), "2432902008176640000");
}

#[test]
pub fn k_tail_factorial() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"x:5;fac:{[a;b]$[a=1;b;fac[a-1;a*b]]};fac[x-1;x]".to_string());
    assert_eq!(format!("{}", i.run(code).unwrap()), "120");
}

#[test]
pub fn k_cond() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"a:{[x;y]$[x y;20;10]};a[{x};10]".to_string());
    assert_eq!(format!("{}", i.run(code).unwrap()), "20");
}

#[test]
pub fn k_cond2() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"a:{[x;y]$[x y;20;10]};a[{x};0]".to_string());
    assert_eq!(format!("{}", i.run(code).unwrap()), "10");

}

#[test]
pub fn k_14() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"f:{a:9};a:14;k:{[x] a}; k 3".to_string());
    assert_eq!(format!("{}", i.run(code).unwrap()), "14");
}


#[test]
pub fn k_multiargs2() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"b:2;a:3;fac:{[x;y]x*y};fac[b*a;a+1]".to_string());
    assert_eq!(format!("{}", i.run(code).unwrap()), "24");
}

#[test]
pub fn k_multiargs() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"xa:9;f:{[x;y;z]x+y*z};f[1;xa+11;3]".to_string());
    assert_eq!(format!("{}", i.run(code).unwrap()), "61");
}

#[test]
pub fn k_repl1() {
    let mut i = Interpreter::new().unwrap();
    let code = i.parse(&"y:3;addy:{[x]y};f:{[g;y]g y};f[addy;1]".to_string());
    assert_eq!(format!("{}", i.run(code).unwrap()), "3");
}
