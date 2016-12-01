
// O-DSL AST

use std::fmt;
use std::iter;
use std::vec;
use std::result::Result;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Error {
    ParseError,
    EvalError { desc: String, ast: AST },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ParseError => write!(f, "Parse error!\n"),
            Error::EvalError { ref desc, ref ast } => {
                write!(f, "Eval error: {}.\nCaused here: {:?}\n", desc, ast)
            }    
        }
    }
}

#[derive(PartialEq,Debug, Clone)]
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

#[derive(PartialEq,Debug,Clone)]
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

impl fmt::Display for Verb {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Verb::Plus => write!(f, "+"),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(PartialEq,Debug,Clone)]
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
            ";:" => Ok(Adverb::Separator),
            "':" => Ok(Adverb::EachPrio),
            ":" => Ok(Adverb::Assign),
            "::" => Ok(Adverb::View),
            "\\:" => Ok(Adverb::EachLeft),
            "/:" => Ok(Adverb::EachRight),
            _ => Err(Error::ParseError),
        }
    }
}

impl fmt::Display for Adverb {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Adverb::Over => write!(f, "/"),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(PartialEq,Debug,Clone)]
pub enum AST {
    // 0
    Nil,
    // 1
    Cons(Box<AST>, Box<AST>),
    // 2
    List(Box<AST>),
    // 3
    Dict(Box<AST>),
    // 4
    Call(Box<AST>, Box<AST>),
    // 5
    Lambda(Box<AST>, Box<AST>),
    // 6
    Verb(Verb, Box<AST>, Box<AST>),
    // 7
    Adverb(Adverb, Box<AST>, Box<AST>),
    // 8
    Ioverb(String),
    // 9
    Name(String),
    // A
    Number(u64),
    // B
    Hexlit(u64),
    // C
    Bool(bool),
    // D
    Symbol(String),
    // E
    Sequence(String),
    // F
    Cell(Box<Cell>),
    // Syntactic sugar
    Assign(Box<AST>, Box<AST>),
    //
    Cond(Box<AST>, Box<AST>, Box<AST>),
}

impl AST {
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
    pub fn len(&self) -> usize {
        match self {
            &AST::List(ref car) => car.len(),
            &AST::Cons(_, ref cdr) => 1 + cdr.len(),
            &AST::Nil => 0,
            _ => 1,
        }
    }
    pub fn is_empty(&self) -> bool {
        self == &AST::Nil
    }
    pub fn is_cons(&self) -> bool {
        match self {
            &AST::Cons(_, _) => true,
            _ => false,
        }
    }
    pub fn shift(self) -> Option<(AST, AST)> {
        match self {
            AST::Cons(car, cdr) => Some((*car, *cdr)),
            AST::Nil => None,
            x => Some((x, AST::Nil)),
        }
    }
    pub fn to_vec(self) -> Vec<AST> {
        let mut out = vec![];
        let mut l = self;
        loop {
            match l.clone() {
                AST::Cons(box car, box cdr) => {
                    out.push(car);
                    l = cdr;
                }
                AST::Nil => break,
                x => out.push(x),
            }
        }
        out
    }
}

impl iter::IntoIterator for AST {
    type Item = AST;
    type IntoIter = vec::IntoIter<AST>;

    fn into_iter(self) -> Self::IntoIter {
        self.to_vec().into_iter()
    }
}


impl fmt::Display for AST {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AST::Nil => write!(f, ""),
            AST::Cons(box ref a, box ref b) => write!(f, "{} {}", a, b),
            AST::List(box ref a) => write!(f, "{}", a),
            AST::Dict(box ref d) => write!(f, "[{};]", d),
            AST::Call(box ref a, box ref b) => write!(f, "{} {}", a, b),
            AST::Lambda(box ref a, box ref b) => {
                match a {
                    &AST::Nil => write!(f, "{{[x]{}}}", b),
                    _ => {
                        let args = format!("{}", a).replace(" ", ";");
                        write!(f, "{{[{}]{}}}", args, b)
                    }
                }
            }
            AST::Verb(ref v, box ref a, box ref b) => write!(f, "{}{}{}", a, v, b),
            AST::Adverb(ref v, box ref a, box ref b) => write!(f, "{}{}{}", a, v, b),
            AST::Ioverb(ref v) => write!(f, "{}", v),
            AST::Name(ref n) => write!(f, "{}", n),
            AST::Number(n) => write!(f, "{}", n),
            AST::Hexlit(h) => write!(f, "0x{}", h),
            AST::Bool(b) => write!(f, "{:?}", b),
            AST::Symbol(ref s) => write!(f, "{}", s),
            AST::Sequence(ref s) => write!(f, "{:?}", s),
            AST::Cell(box ref c) => write!(f, "{}", c),
            AST::Assign(box ref a, box ref b) => write!(f, "{}:{}", a, b),
            AST::Cond(box ref c, box ref a, box ref b) => write!(f, "$[{};{};{}]", c, a, b),
        }

    }
}

#[derive(PartialEq,Debug, Clone)]
pub struct Cell {
    t: Type,
    v: Vec<AST>,
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(");
        for i in &self.v {
            write!(f, "{}", i);
        }
        write!(f, ")")
    }
}

pub fn call(l: AST, r: AST) -> AST {
    AST::Call(l.boxed(), r.boxed())
}

pub fn cons(l: AST, r: AST) -> AST {
    AST::Cons(l.boxed(), r.boxed())
}

pub fn fun(l: AST, r: AST) -> AST {
    AST::Lambda(l.boxed(), r.boxed())
}

pub fn dict(l: AST) -> AST {
    match l {
        AST::Cons(a, b) => AST::Dict(AST::Cons(a, b).boxed()),
        x => x,
    }
}

pub fn list(l: AST) -> AST {
    match l {
        AST::Cons(a, b) => AST::List(AST::Cons(a, b).boxed()),
        x => x,
    }
}

pub fn verb(v: Verb, l: AST, r: AST) -> AST {
    match v {
        Verb::Cast => {
            let rexpr = match r {
                AST::Dict(box d) => {
                    match d {
                        AST::Cons(box a, box b) => {
                            match b {
                                AST::Cons(box t, box f) => {
                                    AST::Cond(a.boxed(), t.boxed(), f.boxed())
                                }
                                x => x,
                            }
                        }
                        x => x,
                    }
                }
                x => x, 
            };
            match l {
                AST::Nil => rexpr,
                _ => AST::Call(l.boxed(), rexpr.boxed()), 
            }
        }
        _ => {
            match r { // optional AST transformations could be done during parsing
                AST::Adverb(a, al, ar) => {
                    match a {
                        Adverb::Assign => AST::Assign(al.boxed(), ar.boxed()),
                        _ => AST::Adverb(a, AST::Verb(v, l.boxed(), AST::Nil.boxed()).boxed(), ar),
                    }
                }
                _ => AST::Verb(v, l.boxed(), r.boxed()),
            }
        }
    }
}

pub fn adverb(a: Adverb, l: AST, r: AST) -> AST {
    match a {
        Adverb::Assign => AST::Assign(l.boxed(), r.boxed()),
        _ => AST::Adverb(a, l.boxed(), r.boxed()),
    }
}
