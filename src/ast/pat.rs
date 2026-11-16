use crate::{
    diagnostic::Span,
    var::{Ident, Operator},
};

pub enum Pat {
    Str(StrPat),
    Int(IntPat),
    Bin(BinPat),
    Wild(WildPat),
    Cons(ConsPat),
    List(ListPat),
    Paren(ParenPat),
    Error(ErrorPat),
}

pub struct StrPat {
    pub str:  String,
    pub span: Span,
}

pub struct IntPat {
    pub int:  i64,
    pub span: Span,
}

pub struct BinPat {
    pub op:   Operator,
    pub lhs:  Box<Pat>,
    pub rhs:  Box<Pat>,
    pub span: Span,
}

pub struct WildPat {
    pub span: Span,
}

pub struct ConsPat {
    pub var:  Ident,
    pub args: Vec<Pat>,
    pub span: Span,
}

pub struct ListPat {
    pub span: Span,
}

pub struct ParenPat {
    pub pat:  Box<Pat>,
    pub span: Span,
}

pub struct ErrorPat {
    pub span: Span,
}

impl Pat {
    pub fn span(&self) -> Span {
        match self {
            Pat::Str(pat) => pat.span,
            Pat::Int(pat) => pat.span,
            Pat::Bin(pat) => pat.span,
            Pat::Wild(pat) => pat.span,
            Pat::Cons(pat) => pat.span,
            Pat::List(pat) => pat.span,
            Pat::Paren(pat) => pat.span,
            Pat::Error(pat) => pat.span,
        }
    }
}
