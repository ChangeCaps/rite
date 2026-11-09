use crate::{diagnostic::Span, var::Ident};

#[derive(Debug)]
pub enum Kind {
    Var(VarKind),
    Arity(ArityKind),
    Error(ErrorKind),
}

#[derive(Debug)]
pub struct VarKind {
    pub var:  Option<Ident>,
    pub span: Span,
}

#[derive(Debug)]
pub struct ArityKind {
    pub arity: usize,
    pub span:  Span,
}

#[derive(Debug)]
pub struct ErrorKind {
    pub span: Span,
}

impl Kind {
    pub fn span(&self) -> Span {
        match self {
            Kind::Var(kind) => kind.span,
            Kind::Arity(kind) => kind.span,
            Kind::Error(kind) => kind.span,
        }
    }
}
