use crate::{
    diagnostic::Span,
    var::{Ident, Operator},
};

pub enum Pat {
    Bin(BinPat),
    Wild(WildPat),
    Cons(ConsPat),
    List(ListPat),
    Paren(ParenPat),
    Error(ErrorPat),
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
    pub var:  Option<Ident>,
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
            Pat::Bin(pat) => pat.span,
            Pat::Wild(pat) => pat.span,
            Pat::Cons(pat) => pat.span,
            Pat::List(pat) => pat.span,
            Pat::Paren(pat) => pat.span,
            Pat::Error(pat) => pat.span,
        }
    }
}
