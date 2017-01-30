
// O-DSL AST

use std::fmt;
use std::iter;
use std::result::Result;
use std::collections::HashMap;
use commands::command;
use streams::otree;
use reactors::task::Context;
use streams::interpreter::*;
use std::cell::UnsafeCell;
use std::isize;
use std::intrinsics::size_of;
use std::rc::Rc;
use core::ops::Deref;
use core::slice::Iter;
use std::mem;

#[derive(Debug)]
pub enum Error {
    ParseError,
    EvalError { desc: String, ast: String },
    InternalError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ParseError => write!(f, "Parse error!\n"),
            Error::EvalError { ref desc, ref ast } => write!(f, "Eval error: {}.\nCaused here: {}\n", desc, ast),
            Error::InternalError => write!(f, "Internal error!\n"),
        }
    }
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

#[derive(PartialEq,Debug,Clone, Copy)]
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
            Verb::Minus => write!(f, "-"),
            Verb::Equal => write!(f, "="),
            Verb::Dot => write!(f, "."),
            Verb::Cast => write!(f, "$"),
            Verb::Times => write!(f, "*"),
            Verb::Divide => write!(f, "%"),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(PartialEq,Debug,Clone, Copy)]
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
pub enum AST<'a> {
    Nil,
    Any,
    List(&'a ASTNode<'a>),
    Dict(&'a ASTNode<'a>),
    Call(&'a ASTNode<'a>, &'a ASTNode<'a>),
    Assign(&'a ASTNode<'a>, &'a ASTNode<'a>),
    Cond(&'a ASTNode<'a>, &'a ASTNode<'a>, &'a ASTNode<'a>),
    Lambda(Option<otree::NodeId>, &'a ASTNode<'a>, &'a ASTNode<'a>),
    Verb(Verb, &'a ASTNode<'a>, &'a ASTNode<'a>),
    Adverb(Adverb, &'a ASTNode<'a>, &'a ASTNode<'a>),
    Table(&'a ASTNode<'a>, &'a ASTNode<'a>),
    Ioverb(String),
    Yield(Context<'a>),
    Value(Value),
    NameInt(u16),
}

#[derive(PartialEq,Debug,Clone)]
pub enum Value {
    Nil,
    SymbolInt(u16),
    SequenceInt(u16),
    Number(i64),
    Float(f64),
    VecInt(Vec<i64>),
    VecFloat(Vec<f64>),
    Ioverb(String),
}

#[derive(PartialEq,Debug,Clone)]
pub enum ASTNode<'a> {
    AST(AST<'a>),
    VecAST(Vec<ASTNode<'a>>),
}

pub type ASTIter<'a> = Iter<'a, ASTNode<'a>>;

#[derive(Debug)]
pub struct Arena<'a> {
    pub names: UnsafeCell<HashMap<String, u16>>,
    pub symbols: UnsafeCell<HashMap<String, u16>>,
    pub sequences: UnsafeCell<HashMap<String, u16>>,
    pub builtins: u16,
    pub asts: UnsafeCell<Vec<ASTNode<'a>>>,
    pub conts: UnsafeCell<Vec<Cont<'a>>>,
}

fn delta<'a>(this: &Cont<'a>, next: &Cont<'a>) -> usize {
    let this_ptr = (this as *const Cont<'a>) as usize;
    let next_ptr = (next as *const Cont<'a>) as usize;
    unsafe { (this_ptr - next_ptr) / size_of::<Cont<'a>>() }
}

fn is_int(x: &ASTNode) -> bool {
    match x {
        &ASTNode::AST(AST::Value(Value::Number(_))) => true,
        _ => false,
    }
}

fn is_float(x: &ASTNode) -> bool {
    match x {
        &ASTNode::AST(AST::Value(Value::Float(_))) => true,
        _ => false,
    }
}

fn fn_false(x: &ASTNode) -> bool {
    false
}

fn is_monovec<'a>(n: &'a Vec<ASTNode<'a>>) -> bool {
    if n.len() == 0 {
        false
    } else {
        // yes, Rust does not like closures in match :(
        let pred = match n[0] {
            ASTNode::AST(AST::Value(Value::Number(_))) => is_int,
            ASTNode::AST(AST::Value(Value::Float(_))) => is_float,
            _ => fn_false,
        };

        let isvec = n.iter().all(pred);
        isvec
    }
}

fn to_monovec<'a, 'b>(n: &'a Vec<ASTNode<'a>>) -> AST<'b> {
    // converts list of integers/floats to specialized vector

    let mut i: Vec<i64> = vec![];
    let mut f: Vec<f64> = vec![];

    for v in n.iter() {
        match v {
            &ASTNode::AST(AST::Value(Value::Number(x))) => {
                i.push(x);
            }
            &ASTNode::AST(AST::Value(Value::Float(x))) => {
                f.push(x);
            }
            _ => panic!("Unexpected non-number"),
        }
    }
    if i.len() >= f.len() {
        AST::Value(Value::VecInt(i))
    } else {
        AST::Value(Value::VecFloat(f))
    }
}

pub fn postprocess_ast<'a, 'b>(n: &'b ASTNode<'a>, skip_depth: i64, arena: &'a Arena<'a>) -> &'a ASTNode<'a> {
    arena.ast(postprocess(n, skip_depth - 1, arena))
}

pub fn postprocess<'a, 'b>(n: &'b ASTNode<'a>, skip_depth: i64, arena: &'a Arena<'a>) -> ASTNode<'a> {
    // AST postprocessing
    // - lists of integers => VecInt
    // - lists of floats   => VecFloat
    // - general lists     => VecAST

    // println!("postprocess input: {:?}", n);

    match n {
        &ASTNode::AST(ref x) => {
            ASTNode::AST(match x {
                &AST::List(l) => AST::List(postprocess_ast(l, skip_depth, arena)),
                &AST::Dict(d) => AST::Dict(postprocess_ast(d, skip_depth, arena)), 
                &AST::Assign(a, b) => {
                    AST::Assign(postprocess_ast(a, skip_depth, arena),
                                postprocess_ast(b, skip_depth, arena))
                } 
                &AST::Call(a, b) => {
                    AST::Call(postprocess_ast(a, skip_depth, arena),
                              postprocess_ast(b, 3 /* Call->Dict->VecAST nodes */, arena))
                } 
                &AST::Cond(a, b, c) => {
                    AST::Cond(postprocess_ast(a, skip_depth, arena),
                              postprocess_ast(b, skip_depth, arena),
                              postprocess_ast(c, skip_depth, arena))
                }
                &AST::Lambda(t, a, b) => {
                    AST::Lambda(t,
                                postprocess_ast(a, skip_depth, arena),
                                postprocess_ast(b, 2 /* Lambda->VecAST nodes */, arena))
                }
                &AST::Verb(v, a, b) => {
                    AST::Verb(v,
                              postprocess_ast(a, skip_depth, arena),
                              postprocess_ast(b, skip_depth, arena))
                }
                &AST::Adverb(adv, a, b) => {
                    AST::Adverb(adv,
                                postprocess_ast(a, skip_depth, arena),
                                postprocess_ast(b, skip_depth, arena))
                }
                &AST::Table(a, b) => {
                    AST::Table(postprocess_ast(a, skip_depth, arena),
                               postprocess_ast(b, skip_depth, arena))
                }
                x => x.clone(),  // nothing to postprocess
            })
        }
        &ASTNode::VecAST(ref x) => {
            if (skip_depth <= 0) && is_monovec(x) {
                ASTNode::AST(to_monovec(x))
            } else {
                let v = x.iter().map(|x| postprocess(x, skip_depth - 1, arena)).collect();
                ASTNode::VecAST(v)
            }
        }
    }
}

pub fn parse<'a>(arena: &'a Arena<'a>, s: &String) -> &'a ASTNode<'a> {
    let ast = command::parse_Mex(arena, s).unwrap();
    // println!("parse {:?}", ast);
    let p_ast = postprocess(&ast, 0, arena);
    // println!("post parse {:?}", p_ast);
    arena.ast(p_ast)
}

struct viter<'a>(&'a ASTIter<'a>);
impl<'a> fmt::Display for viter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "iter")
    }
}

impl<'a> fmt::Display for Cont<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Cont::Call(callee, cont) => write!(f, "call: {} next: {}", callee, delta(self, cont)),
            &Cont::Func(names, args, body, cont) => write!(f, "func: {} {} next: {}", names, args, delta(self, cont)),
            &Cont::Cond(if_expr, else_expr, cont) => {
                write!(f,
                       "cond: {} {} next: {}",
                       if_expr,
                       else_expr,
                       delta(self, cont))
            }
            &Cont::Assign(name, cont) => write!(f, "assign: {} next: {}", name, delta(self, cont)),
            &Cont::List(_, _, cont) => write!(f, "list: NYI next: {}", delta(self, cont)),
            &Cont::Dict(ref acc, ref rest, cont) => {
                write!(f,
                       "dict: {} {} next: {}",
                       acc,
                       viter(rest),
                       delta(self, cont))
            }
            &Cont::DictComplete(ref acc, ref rest, idx, cont) => {
                write!(f,
                       "dict_complete: {} {} next: {}",
                       acc,
                       viter(rest),
                       delta(self, cont))
            }
            &Cont::Adverb(adv, _, cont) => write!(f, "adverb: {} next: {}", adv, delta(self, cont)),
            &Cont::Verb(ref verb, right, swap, cont) => {
                write!(f, "verb: {} {} next: {}", verb, right, delta(self, cont))
            }
            &Cont::Expressions(ast, Some(ref rest), cont) => {
                write!(f,
                       "ast: {} expr: {} next: {}",
                       ast,
                       viter(rest),
                       delta(self, cont))
            }
            &Cont::Expressions(ast, None, cont) => write!(f, "ast: {} next: {}", ast, delta(self, cont)),
            &Cont::Return => write!(f, "return"),
            &Cont::Intercore(ref msg, cont) => write!(f, "intercore: {:?} {}", msg.clone(), delta(self, cont)),
            &Cont::Yield(cont) => write!(f, "yield: {}", delta(self, cont)),
        }
    }
}

impl<'a> fmt::Display for Lazy<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Lazy::Defer(node, ast, cont) => write!(f, "defer {:?} {} {}", node, ast, cont),
            x => write!(f, "return"),
        }
    }
}

impl<'a> Arena<'a> {
    pub fn new() -> Arena<'a> {
        let (builtins, asts) = Arena::init(UnsafeCell::new(Vec::with_capacity(2048 * 2048)));
        Arena {
            asts: asts,
            names: UnsafeCell::new(HashMap::new()),
            symbols: UnsafeCell::new(HashMap::new()),
            sequences: UnsafeCell::new(HashMap::new()),
            conts: UnsafeCell::new(Vec::with_capacity(2048 * 2048)),
            builtins: builtins,
        }
    }

    pub fn nil(&'a self) -> &'a ASTNode<'a> {
        unsafe { &(*self.asts.get())[0] } // see Arena::init for details
    }

    pub fn any(&'a self) -> &'a ASTNode<'a> {
        unsafe { &(*self.asts.get())[1] } // see Arena::init for details
    }

    pub fn yield_(&'a self) -> &'a ASTNode<'a> {
        unsafe { &(*self.asts.get())[2] } // see Arena::init for details
    }

    pub fn valnil(&'a self) -> &'a ASTNode<'a> {
        unsafe { &(*self.asts.get())[3] } // see Arena::init for details
    }

    pub fn dump(&'a self) {
        let x = unsafe { &mut *self.asts.get() };
        for i in x.iter() {
            println!("ast {}", i);
        }
        let x = unsafe { &mut *self.conts.get() };
        for i in x.iter() {
            println!("cont {}", i);
        }
    }

    pub fn ast(&self, n: ASTNode<'a>) -> &'a ASTNode<'a> {
        let ast = unsafe { &mut *self.asts.get() };
        ast.push(n);
        ast.last().unwrap()
    }

    pub fn vec(&self, v: Vec<ASTNode<'a>>) -> &'a ASTNode<'a> {
        self.ast(ASTNode::VecAST(v))
    }

    pub fn cont(&self, n: Cont<'a>) -> &'a Cont<'a> {
        let conts = unsafe { &mut *self.conts.get() };
        conts.push(n);
        conts.last().unwrap()
    }

    pub fn intern(&self, s: String) -> ASTNode<'a> {
        let names = unsafe { &mut *self.names.get() };

        ASTNode::AST(if names.contains_key(&s) {
            AST::NameInt(names[&s])
        } else {
            let id = names.len() as u16;
            names.insert(s, id);
            AST::NameInt(id)
        })
    }

    pub fn intern_ast(&self, s: String) -> &'a ASTNode<'a> {
        self.ast(self.intern(s))
    }

    pub fn intern_symbol(&self, s: String) -> ASTNode<'a> {
        let symbols = unsafe { &mut *self.symbols.get() };

        ASTNode::AST(if symbols.contains_key(&s) {
            AST::Value(Value::SymbolInt(symbols[&s]))
        } else {
            let id = symbols.len() as u16;
            symbols.insert(s, id);
            AST::Value(Value::SymbolInt(id))
        })
    }

    pub fn intern_symbol_ast(&self, s: String) -> &'a ASTNode<'a> {
        self.ast(self.intern_symbol(s))
    }

    pub fn intern_sequence(&self, s: String) -> ASTNode<'a> {
        let sequences = unsafe { &mut *self.sequences.get() };

        ASTNode::AST(if sequences.contains_key(&s) {
            AST::Value(Value::SequenceInt(sequences[&s]))
        } else {
            let id = sequences.len() as u16;
            sequences.insert(s, id);
            AST::Value(Value::SequenceInt(id))
        })
    }

    pub fn to_string(&self) {
        let ast = unsafe { &mut *self.asts.get() };
        println!("AST {}, {:?}", ast.len(), ast);
    }

    pub fn init(asts: UnsafeCell<Vec<ASTNode<'a>>>) -> (u16, UnsafeCell<Vec<ASTNode<'a>>>) {
        let a = unsafe { &mut *asts.get() };
        assert!(a.len() == 0);
        a.push(ASTNode::AST(AST::Value(Value::Nil)));     // Value Nil - index 3
        a.push(ASTNode::AST(AST::Any));                   // Any       - index 1
        a.push(ASTNode::AST(AST::Yield(Context::Nil)));   // Yield     - index 2
        a.push(ASTNode::AST(AST::Nil));                   // Nil       - index 0
        (a.len() as u16, asts)
    }

    pub fn clean(&self) -> usize {
        let asts = unsafe { &mut *self.asts.get() };
        let conts = unsafe { &mut *self.conts.get() };
        let l = conts.len() + asts.len();
        unsafe {
            asts.set_len(self.builtins as usize);
            conts.set_len(0);
        };
        l
    }
}

#[derive(Debug)]
pub struct ASTIntoIterator<'a> {
    // iterator to uniformly iterate over ASTNode::AST/ASTNode::VecAST
    node: &'a ASTNode<'a>,
    idx: i64,
    done: bool,
}

impl<'a> Iterator for ASTIntoIterator<'a> {
    type Item = &'a ASTNode<'a>;
    fn next(&mut self) -> Option<&'a ASTNode<'a>> {
        if self.done {
            return None;
        }

        match self.node {
            &ASTNode::AST(ref x) => {
                self.done = true;
                Some(self.node)
            }
            &ASTNode::VecAST(ref x) => {
                self.idx = self.idx + 1;
                if self.idx as usize >= x.len() {
                    self.done = true;
                    None
                } else {
                    Some(&x[self.idx as usize])
                }
            }
        }
    }
}

impl<'a> iter::IntoIterator for &'a ASTNode<'a> {
    type Item = &'a ASTNode<'a>;
    type IntoIter = ASTIntoIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        ASTIntoIterator {
            node: &self,
            idx: -1,
            done: false,
        }
    }
}

struct vi64<'a>(&'a Vec<i64>);

impl<'a> fmt::Display for vi64<'a> {
    // cannot implement trait directly for Vec<i64> :(
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = self.0
            .into_iter()
            .map(|x| x.to_string())
            .fold(String::new(), |acc, x| if acc == "" {
                x
            } else {
                format!("{};{}", acc, x)
            });
        write!(f, "{}", str)
    }
}

struct vf64<'a>(&'a Vec<f64>);

impl<'a> fmt::Display for vf64<'a> {
    // cannot implement trait directly for Vec<f64> :(
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = self.0
            .into_iter()
            .map(|x| x.to_string())
            .fold(String::new(), |acc, x| if acc == "" {
                x
            } else {
                format!("{};{}", acc, x)
            });
        write!(f, "{}", str)
    }
}

struct vast<'a>(&'a Vec<ASTNode<'a>>);

impl<'a> fmt::Display for vast<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = self.0
            .into_iter()
            .map(|x| x.to_string())
            .fold(String::new(), |acc, x| if acc == "" {
                x
            } else {
                format!("{};{}", acc, x)
            });
        write!(f, "{}", str)
    }
}

impl<'a> fmt::Display for ASTNode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ASTNode::AST(AST::Any) => write!(f, "Any"),
            ASTNode::AST(AST::List(ref a)) => write!(f, "l({})", a),
            ASTNode::AST(AST::Table(a, b)) => write!(f, "t([{}]{})", a, b),
            ASTNode::AST(AST::Dict(ref d)) => write!(f, "d[{}]", d),
            ASTNode::AST(AST::Call(ref a, ref b)) => write!(f, "{} {}", a, b),
            ASTNode::AST(AST::Lambda(_, a, b)) => {
                match *a {
                    ASTNode::AST(AST::Value(Value::Nil)) => write!(f, "{{[x]{}}}", b),
                    _ => {
                        let args = format!("{}", a).replace(" ", ";");
                        write!(f, "{{[{}]{}}}", args, b)
                    }
                }
            }
            ASTNode::AST(AST::Verb(ref v, ref a, ref b)) => write!(f, "{}{}{}", a, v, b),
            ASTNode::AST(AST::Adverb(ref v, ref a, ref b)) => write!(f, "{}{}{}", a, v, b),
            ASTNode::AST(AST::Assign(ref a, ref b)) => write!(f, "{}:{}", a, b),
            ASTNode::AST(AST::Cond(ref c, ref a, ref b)) => write!(f, "$[{};{};{}]", c, a, b),
            ASTNode::AST(AST::Yield(ref c)) => write!(f, "Yield {:?}", c),
            ASTNode::AST(AST::NameInt(ref n)) => write!(f, "^{}", n),
            ASTNode::AST(AST::Value(ref v)) => {
                match v {
                    &Value::Nil => write!(f, "Nil"),
                    &Value::Number(n) => write!(f, "{}", n),
                    &Value::SymbolInt(ref s) => write!(f, "{}", s),
                    &Value::SequenceInt(ref s) => write!(f, "{:?}s", s),
                    &Value::VecInt(ref v) => write!(f, "#i[{}]", vi64(v)),
                    &Value::VecFloat(ref v) => write!(f, "#f[{}]", vf64(v)),
                    &Value::Ioverb(ref v) => write!(f, "{}", v),
                    _ => write!(f, "Not implemented yet."),
                }
            }
            ASTNode::VecAST(ref v) => write!(f, "#a[{}]", vast(v)),
            _ => write!(f, "Not implemented yet."),
        }

    }
}

pub fn extract_name<'a>(a: &'a ASTNode<'a>) -> u16 {
    match a {
        &ASTNode::AST(AST::NameInt(s)) => s,
        _ => 0,
    }
}
pub fn call<'a>(l: &'a ASTNode<'a>, r: &'a ASTNode<'a>, arena: &'a Arena<'a>) -> ASTNode<'a> {
    ASTNode::AST(AST::Call(l, r))
}

pub fn fun<'a>(l: &'a ASTNode<'a>, r: &'a ASTNode<'a>, arena: &'a Arena<'a>) -> ASTNode<'a> {
    ASTNode::AST(match *l {
        ASTNode::AST(AST::Value(Value::Nil)) => AST::Lambda(None, arena.intern_ast("x".to_string()), r),
        _ => AST::Lambda(None, l, r),
    })
}

pub fn table<'a>(l: &'a ASTNode<'a>, r: &'a ASTNode<'a>, arena: &'a Arena<'a>) -> ASTNode<'a> {
    ASTNode::AST(AST::Table(l, r))
}

pub fn dict<'a>(l: &'a ASTNode<'a>, arena: &'a Arena<'a>) -> ASTNode<'a> {
    ASTNode::AST(AST::Dict(l))
}

pub fn list<'a>(l: &'a ASTNode<'a>, arena: &'a Arena<'a>) -> ASTNode<'a> {
    ASTNode::AST(AST::List(l))
}

pub fn verb<'a>(v: Verb, l: &'a ASTNode<'a>, r: &'a ASTNode<'a>, arena: &'a Arena<'a>) -> ASTNode<'a> {
    match v {
        Verb::Dot => {
            ASTNode::AST(match (l, r) {
                (&ASTNode::AST(AST::Value(Value::Number(x))), &ASTNode::AST(AST::Value(Value::Number(y)))) => {
                    AST::Value(Value::Float((x as f64) +
                                            (format!("0.{}", y)).parse::<f64>().expect("Invalid fraction")))
                }
                _ => AST::Verb(v, l, r),
            })
        }
        Verb::Cast => {
            let rexpr = match r {
                &ASTNode::AST(AST::Dict(&ASTNode::VecAST(ref x))) if x.len() == 3 => AST::Cond(&x[0], &x[1], &x[2]),
                _ => AST::Verb(v, l, r),
            };
            ASTNode::AST(match *l {
                ASTNode::AST(AST::Value(Value::Nil)) => rexpr,
                _ => AST::Call(l, arena.ast(ASTNode::AST(rexpr))), 
            })

        }
        _ => {
            ASTNode::AST(match r { // optional AST transformations could be done during parsing
                &ASTNode::AST(AST::Adverb(ref a, al, ar)) => {
                    match a {
                        &Adverb::Assign => AST::Assign(al, ar),
                        x => {
                            AST::Adverb(x.clone(),
                                        arena.ast(ASTNode::AST(AST::Verb(v, l, arena.nil()))),
                                        ar)
                        }
                    }
                }
                _ => AST::Verb(v, l, r),
            })
        }
    }
}

pub fn adverb<'a>(a: Adverb, l: &'a ASTNode<'a>, r: &'a ASTNode<'a>, arena: &'a Arena<'a>) -> ASTNode<'a> {
    ASTNode::AST(match a {
        Adverb::Assign => AST::Assign(l, r),
        _ => AST::Adverb(a, l, r),
    })
}

#[derive(Debug)]
pub struct ASTAcc<'a> {
    // mutable vector for accumulating calculation results + having multiple immutable references to it
    inner: Rc<UnsafeCell<Vec<ASTNode<'a>>>>,
}

impl<'a> Clone for ASTAcc<'a> {
    fn clone(&self) -> ASTAcc<'a> {
        unsafe { ASTAcc { inner: self.inner.clone() } }
    }
}

impl<'a> ASTAcc<'a> {
    pub fn new() -> ASTAcc<'a> {
        let a: Vec<ASTNode<'a>> = Vec::new();
        ASTAcc { inner: Rc::new(UnsafeCell::new(a)) }
    }

    pub fn push(&'a self, n: &'a ASTNode<'a>) -> usize {
        unsafe {
            (*self.inner.deref().get()).push(n.clone());
            self.len() - 1
        }
    }

    pub fn set(&'a self, idx: usize, val: ASTNode<'a>) {
        unsafe {
            (&mut *self.inner.deref().get())[idx] = val;
        }
    }

    pub fn get(&'a self) -> &'a Vec<ASTNode<'a>> {
        unsafe { &*self.inner.deref().get() }
    }

    pub fn len(&'a self) -> usize {
        unsafe { (*self.inner.deref().get()).len() }
    }

    pub fn disown(&'a self) -> Vec<ASTNode<'a>> {
        // transfer vector ownership
        unsafe { mem::replace(&mut *self.inner.deref().get(), vec![]) }
    }
}

impl<'a> fmt::Display for ASTAcc<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            let v = &*self.inner.deref().get();
            let str = v.iter()
                .map(|x| x.to_string())
                .fold(String::new(), |acc, x| if acc == "" {
                    x
                } else {
                    format!("{};{}", acc, x)
                });
            write!(f, "[{}]", str)
        }
    }
}