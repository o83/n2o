extern crate kernel;

use kernel::commands::*;
use kernel::commands::ast::*;

#[test]
pub fn k_ariph() {
    assert_eq!(format!("{:?}", command::parse_Mex("1+2")),
               "Ok(Stmt(Verb(Plus), Number(1), Number(2)))");

    assert_eq!(format!("{:?}", command::parse_Mex("1+2*4")),
               "Ok(Stmt(Verb(Plus), Number(1), Stmt(Verb(Times), Number(2), Number(4))))");
}

#[test]
pub fn k_list() {
    assert_eq!(format!("{:?}", command::parse_Mex("(1;2;3;4)")),
               "Ok(List(Cons(Number(1), Cons(Number(2), Cons(Number(3), Number(4))))))");
}

#[test]
pub fn k_assign() {
    assert_eq!(format!("{:?}", command::parse_Mex("a:b:c:1")),
               "Ok(Assign(Name(\"a\"), Assign(Name(\"b\"), Assign(Name(\"c\"), Number(1)))))");
}

#[test]
pub fn k_func() {
    assert_eq!(format!("{:?}", command::parse_Mex("{x*2}[(1;2;3)]")),
               "Ok(Call(Lambda(Nil, Stmt(Verb(Times), Name(\"x\"), Number(2))), \
                List(Cons(Number(1), Cons(Number(2), Number(3))))))");
}

#[test]
pub fn k_adverb() {
    assert_eq!(format!("{:?}", command::parse_Mex("{x+2}/(1;2;3)")),
               "Ok(Stmt(Adverb(Over), Lambda(Nil, Stmt(Verb(Plus), Name(\"x\"), Number(2))), \
                List(Cons(Number(1), Cons(Number(2), Number(3))))))");
}


#[test]
pub fn k_reduce() {
    assert_eq!(format!("{:?}",
                       command::parse_Mex("+/{x*y}[(1;3;4;5;6);(2;6;2;1;3)]")),
               "Ok(Stmt(Adverb(Over), Verb(Plus), Call(Lambda(Nil, Stmt(Verb(Times), \
                Name(\"x\"), Name(\"y\"))), Cons(List(Cons(Number(1), Cons(Number(3), \
                Cons(Number(4), Cons(Number(5), Number(6)))))), List(Cons(Number(2), \
                Cons(Number(6), Cons(Number(2), Cons(Number(1), Number(3))))))))))");
}
