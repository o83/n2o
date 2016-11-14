
// O-DSL AST

use std::result::Result;
use std::rc::Rc;

#[derive(Debug)]
pub enum Error {
    ParseError,
}

#[derive(Debug)]
pub enum Type {
    Nil = 0,
    Number = 1,
    Char = 2,
    Symbol = 3,
    List = 4,
    Dictionary = 5,
    Function = 6,
    View = 7,
    NameRef = 8,
    Verb = 9,
    Adverb = 10,
    Return = 11,
    Cond = 12,
    Native = 13,
    Quote = 14,
}

#[derive(Debug)]
pub struct Cell {
    t: Type,
    v: Vec<Cell>,
}

// OK LANG

//        a          l           a-a         l-a         a-l         l-l         triad    tetrad
// "+" : [ident,     flip,       ad(plus),   ad(plus),   ad(plus),   ad(plus),   null,    null  ],
// "-" : [am(negate),am(negate), ad(minus),  ad(minus),  ad(minus),  ad(minus),  null,    null  ],
// "*" : [first,     first,      ad(times),  ad(times),  ad(times),  ad(times),  null,    null  ],
// "%" : [sqrt,      am(sqrt),   ad(divide), ad(divide), ad(divide), ad(divide), null,    null  ],
// "!" : [iota,      odometer,   mod,        md,         ar(mod),    md,         null,    null  ],
// "&" : [where,     where,      ad(min),    ad(min),    ad(min),    ad(min),    null,    null  ],
// "|" : [rev,       rev,        ad(max),    ad(max),    ad(max),    ad(max),    null,    null  ],
// "<" : [asc,       asc,        ad(less),   ad(less),   ad(less),   ad(less),   null,    null  ],
// ">" : [desc,      desc,       ad(more),   ad(more),   ad(more),   ad(more),   null,    null  ],
// "=" : [imat,      group,      ad(equal),  ad(equal),  ad(equal),  ad(equal),  null,    null  ],
// "~" : [am(not),   am(not),    match,      match,      match,      match,      null,    null  ],
// "," : [enlist,    enlist,     cat,        cat,        cat,        cat,        null,    null  ],
// "^" : [pisnull,   am(pisnull),except,     except,     except,     except,     null,    null  ],
// "#" : [count,     count,      take,       reshape,    take,       reshape,    null,    null  ],
// "_" : [am(floor), am(floor),  drop,       ddrop,      drop,       cut,        null,    null  ],
// "$" : [kfmt,      am(kfmt),   dfmt,       dfmt,       dfmt,       dfmt,       null,    null  ],
// "?" : [real,      unique,     rnd,        pfind,      rnd,        ar(pfind),  splice,  null  ],
// "@" : [type,      type,       atd,        atl,        atd,        ar(atl),    amend4,  amend4],
// "." : [keval,     keval,      call,       call,       call,       call,       dmend4,  dmend4],
// "'" : [null,      null,       null,       atl,        kwindow,    ar(atl),    null,    null  ],
// "/" : [null,      null,       null,       null,       pack,       pack,       null,    null  ],
// "\\": [null,      null,       null,       unpack,     split,      null,       null,    null  ],

#[derive(Debug)]
pub enum Verb {
    Plus = 0,
    Minus = 1,
    Times = 2,
    Divide = 3,
    Mod = 4,
    Min = 5,
    Max = 6,
    Less = 7,
    More = 8,
    Equal = 9,
    Match = 10,
    Concat = 11,
    Except = 12,
    Take = 13,
    Drop = 14,
    Cast = 15,
    Find = 16,
    At = 17,
    Dot = 18,
    Gets = 19,
    Pack = 20,
    Unpack = 21,
    New = 22,
}

#[derive(Debug)]
pub enum Monadic {
    Flip = 0,
    Negate = 1,
    First = 2,
    Sqrt = 3,
    Iota = 4,
    Where = 5,
    Rev = 6,
    Asc = 7,
    Desc = 8,
    Group = 9,
    Not = 10,
    List = 11,
    Nil = 12,
    Count = 13,
    Floor = 14,
    Fmt = 15,
    Unique = 16,
    Type = 17,
    Eval = 18,
}

impl Verb {
    pub fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "+" => Ok(Verb::Plus),
            "-" => Ok(Verb::Minus),
            "*" => Ok(Verb::Times),
            "%" => Ok(Verb::Divide),
            "!" => Ok(Verb::Mod),
            "&" => Ok(Verb::Min),
            "|" => Ok(Verb::Max),
            "<" => Ok(Verb::Less),
            ">" => Ok(Verb::More),
            "=" => Ok(Verb::Equal),
            "~" => Ok(Verb::Match),
            "," => Ok(Verb::Concat),
            "^" => Ok(Verb::Except),
            "#" => Ok(Verb::Take),
            "_" => Ok(Verb::Drop),
            "$" => Ok(Verb::Cast),
            "?" => Ok(Verb::Find),
            "@" => Ok(Verb::At),
            "." => Ok(Verb::Dot),
            ";" => Ok(Verb::New),
            _ => Err(Error::ParseError),
        }
    }
}

#[derive(Debug)]
pub enum Adverb {
    Each,
    EachPrio,
    EachLeft,
    EachRight,
    Over,
    Scan,
    Iterate,
    Fixed,
    Assign,
    View,
    Separator,
}

impl Adverb {
    pub fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "/" => Ok(Adverb::Over),
            "\\" => Ok(Adverb::Scan),
            "'" => Ok(Adverb::Each),
            ";" => Ok(Adverb::Separator),
            "':" => Ok(Adverb::EachPrio),
            ":" => Ok(Adverb::Assign),
            "::" => Ok(Adverb::View),
            "\\:" => Ok(Adverb::EachLeft),
            "/:" => Ok(Adverb::EachRight),
            _ => Err(Error::ParseError),
        }
    }
}

#[derive(Debug)]
pub enum Token {
    Space,
    Assign,
    Semi,
    Colon,
    View,
    Cond,
    Apply,
    OpenB,
    OpenP,
    OpenC,
    CloseB,
    CloseP,
    CloseC,
}


#[derive(Debug)]
pub enum AST {
    Number(u64),
    Hexlit(u64),
    Bool(bool),
    Name(String),
    Symbol(String),
    Sequence(String),
    Verb(Verb, Box<AST>, Box<AST>),
    Ioverb(String, Box<AST>),
    Adverb(Adverb, Box<AST>, Box<AST>),
    Adverb2(Adverb, Verb, Box<AST>),
    Assign(Box<AST>, Box<AST>),
    View(Box<AST>, Box<AST>),
    List(Box<AST>),
    Dict(Box<AST>),
    Call(Box<AST>, Box<AST>),
    Lambda(Box<AST>, Box<AST>),
    Nil,
    CommaList(Box<AST>),
    ColonList(Box<AST>),
    DictCons(Box<AST>, Box<AST>),
    Cons(Box<AST>, Box<AST>),
}

pub fn call(l: AST, r: AST) -> AST {
    return AST::Call(Box::new(l), Box::new(r));
}

pub fn cons(l: AST, r: AST) -> AST {
    return AST::Cons(Box::new(l), Box::new(r));
}

pub fn assign(l: AST, r: AST) -> AST {
    return AST::Assign(Box::new(l), Box::new(r));
}

pub fn fun(l: AST, r: AST) -> AST {
    return AST::Lambda(Box::new(l), Box::new(r));
}

pub fn dict(l: AST) -> AST {
    return AST::Dict(Box::new(l));
}

pub fn list(l: AST) -> AST {
    return AST::List(Box::new(l));
}

pub fn verb(v: Verb, l: AST, r: AST) -> AST {
    return AST::Verb(v, Box::new(l), Box::new(r));
}

pub fn adverb(v: Adverb, l: AST, r: AST) -> AST {
    return AST::Adverb(v, Box::new(l), Box::new(r));
}

pub fn adverb2(a: Adverb, v: Verb, r: AST) -> AST {
    return AST::Adverb2(a, v, Box::new(r));
}
