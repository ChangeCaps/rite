use crate::{diagnostic::Span, var::Ident};

pub enum Pat {
    Cons(ConsPat),
    Error(ErrorPat),
}

pub struct ConsPat {
    pub var:  Option<Ident>,
    pub args: Vec<Pat>,
    pub span: Span,
}

pub struct ErrorPat {
    pub span: Span,
}

impl Pat {
    pub fn span(&self) -> Span {
        match self {
            Pat::Cons(pat) => pat.span,
            Pat::Error(pat) => pat.span,
        }
    }
}
