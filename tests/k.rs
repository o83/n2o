extern crate kernel;

use kernel::commands::*;
use kernel::commands::ast::*;

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
    Ok(AST::Assign(
                   AST::Name(String::from("a")).boxed(),
                   AST::Assign(
                               AST::Name(String::from("b")).boxed(),
                               AST::Assign(
                                           AST::Name(String::from("c")).boxed(),
                                           AST::Number(1).boxed()).boxed()).boxed())));
}

#[test]
pub fn k_func() {
    assert_eq!(format!("{:?}", command::parse_Mex("{x*2}[(1;2;3)]")),
    "Ok(Call(Lambda(Nil, Verb(Times, Name(\"x\"), Number(2))), List(Cons(Number(1), \
    Cons(Number(2), Number(3))))))");
}

#[test]
pub fn k_mini_spec() {
    assert_eq!(format!("{:?}",
                       command::parse_Mex("();[];{};(());[[]];{{}};()();1 2 3;(1 2 3);[1 2 \
                                          3];[a[b[c[d]]]];(a(b(c(d))));{a{b{c{d}}}};")),
                                          "Ok(Cons(Nil, Cons(Nil, Cons(Lambda(Nil, Nil), Cons(Nil, Cons(Nil, \
                                          Cons(Lambda(Nil, Lambda(Nil, Nil)), Cons(Call(Nil, Nil), Cons(Call(Number(1), \
                Call(Number(2), Number(3))), Cons(Call(Number(1), Call(Number(2), Number(3))), \
                                                  Cons(Call(Number(1), Call(Number(2), Number(3))), Cons(Call(Name(\"a\"), \
                Call(Name(\"b\"), Call(Name(\"c\"), Name(\"d\")))), Cons(Call(Name(\"a\"), \
                Call(Name(\"b\"), Call(Name(\"c\"), Name(\"d\")))), Cons(Lambda(Nil, \
                Call(Name(\"a\"), Lambda(Nil, Call(Name(\"b\"), Lambda(Nil, Call(Name(\"c\"), \
                Lambda(Nil, Name(\"d\")))))))), Nil))))))))))))))");
}

#[test]
pub fn k_adverb() {
    assert_eq!(format!("{:?}", command::parse_Mex("{x+2}/(1;2;3)")),
    "Ok(Adverb(Over, Lambda(Nil, Verb(Plus, Name(\"x\"), Number(2))), \
    List(Cons(Number(1), Cons(Number(2), Number(3))))))");
}


#[test]
pub fn k_reduce() {
    assert_eq!(format!("{:?}",
                       command::parse_Mex("+/{x*y}[(1;3;4;5;6);(2;6;2;1;3)]")),
                       "Ok(Adverb(Over, Verb(Plus, Nil, Nil), Call(Lambda(Nil, Verb(Times, Name(\"x\"), \
                       Name(\"y\"))), Dict(Cons(List(Cons(Number(1), Cons(Number(3), Cons(Number(4), \
                Cons(Number(5), Number(6)))))), List(Cons(Number(2), Cons(Number(6), \
                                                                          Cons(Number(2), Cons(Number(1), Number(3)))))))))))");
}
