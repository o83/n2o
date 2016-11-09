
// K primitives: http://kparc.com/lisp.txt

pub enum KType {
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

static ASCII: [[i32; 16]; 16] = [[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                 [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                 [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                 [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                 [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                 [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                 [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                 [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                 [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                 [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                 [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                 [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                 [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                 [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                 [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                 [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]];

pub struct KCell {
    t: KType,
    v: Vec<KCell>,
}

static NUMBERS: [[i32; 8]; 2] = [[1, 2, 3, 4, 5, 6, 7, 8], [1, 2, 3, 4, 5, 6, 7, 8]];

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

pub enum KVerb {
    Plus(Box<AST>, Box<AST>),
    Minus(Box<AST>, Box<AST>),
    Times(Box<AST>, Box<AST>),
    Divide(Box<AST>, Box<AST>),
    Mod(Box<AST>, Box<AST>),
    Max(Box<AST>, Box<AST>),
    Min(Box<AST>, Box<AST>),
    Less(Box<AST>, Box<AST>),
    More(Box<AST>, Box<AST>),
    Equal(Box<AST>, Box<AST>),
    Join(Box<AST>, Box<AST>),
    Ident(Box<AST>),
    Negate(Box<AST>),
    First(Box<AST>),
    Sqrt(Box<AST>),
    Keys(Box<AST>),
    Rev(Box<AST>),
    Desc(Box<AST>),
    Not(Box<AST>),
    Enlist(Box<AST>),
    Nil(Box<AST>),
    Count(Box<AST>),
    Floor(Box<AST>),
    Type(Box<AST>),
    Format(Box<AST>),
    Real(Box<AST>),
    Iota(Box<AST>),
    Eval(Box<AST>),
    Except(Box<AST>),
    Drop(Box<AST>),
    Take(Box<AST>),
    Reshape(Box<AST>),
    Match(Box<AST>),
    Find(Box<AST>),
    Cut(Box<AST>),
    Rnd(Box<AST>),
    Flip(Box<AST>),
    Asc(Box<AST>),
    Where(Box<AST>),
    Group(Box<AST>),
    Unique(Box<AST>),
    Split(Box<AST>),
    Pack(Box<AST>),
    Unpack(Box<AST>),
    Splice(Box<AST>),
    Window(Box<AST>),
    Odometer(Box<AST>),
    Imat(Box<AST>),
}

pub enum KAdverbs {
    Each,
    Over,
    Scan,
    Fixed,
}

#[derive(Debug)]
pub enum AST {
    Integer(u64),
    Symbol(String),
    Float(f64),
    Append,
    Get,
    Set,
    Curry,
    Compose,
    Lambda(Box<AST>, Box<AST>),
    Expr,
    Nil,
    Call(Box<AST>, Box<AST>),
    CommaList(Box<AST>),
    ColonList(Box<AST>),
    Cons(Box<AST>, Box<AST>),
    Car,
    Setq,
    Cond,
    Map,
    Reduce(Box<AST>),
    Min,
    Max,
    Greater,
    Less,
    Equal,
    Length,
    Reverse,
    Member,
    Plus(Box<AST>, Box<AST>),
    Minus(Box<AST>, Box<AST>),
    Times(Box<AST>, Box<AST>),
    Divide(Box<AST>, Box<AST>),
}
