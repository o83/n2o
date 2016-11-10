extern crate kernel;

use kernel::commands::*;
use kernel::commands::ast::*;

#[test]
pub fn k_ariph() {
    assert_eq!(format!("{:?}", command::parse_Mex("1+2")),
               "Ok(Sentence(Number(1), Verb(Plus), Number(2)))");

    assert_eq!(format!("{:?}", command::parse_Mex("1+2*4")),
               "Ok(Sentence(Sentence(Number(1), Verb(Plus), Number(2)), Verb(Times), Number(4)))");
}

#[test]
pub fn k_list() {
    assert_eq!(format!("{:?}", command::parse_Mex("(1;2;3;4)")),
               "Ok(List(Cons(Number(1), Cons(Number(2), Cons(Number(3), Cons(Number(4), Nil))))))");
}

#[test]
pub fn k_func() {
    assert_eq!(format!("{:?}", command::parse_Mex("{x*2}[(1;2;3)]")),
               "Ok(Call(Lambda(Nil, Sentence(Name(\"x\"), Verb(Times), Number(2))), \
                Cons(List(Cons(Number(1), Cons(Number(2), Cons(Number(3), Nil)))), Nil)))");
}

#[test]
pub fn k_adverb() {
    assert_eq!(format!("{:?}", command::parse_Mex("{x+2}/(1;2;3)")),
               "Ok(Sentence(Adverb(Over), Lambda(Nil, Sentence(Name(\"x\"), Verb(Plus), \
                Number(2))), List(Cons(Number(1), Cons(Number(2), Cons(Number(3), Nil))))))");
}
