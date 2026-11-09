use crate::{ast::Kind, diagnostic::Span, var::Ident};

#[derive(Debug)]
pub enum Ty {
    Var(VarTy),
    Lam(LamTy),
    App(AppTy),
    Where(WhereTy),
    Error(ErrorTy),
}

#[derive(Debug)]
pub struct VarTy {
    pub var:  Option<Ident>,
    pub span: Span,
}

#[derive(Debug)]
pub struct LamTy {
    pub input:  Box<Ty>,
    pub output: Box<Ty>,
    pub span:   Span,
}

#[derive(Debug)]
pub struct AppTy {
    pub lambda: Box<Ty>,
    pub input:  Box<Ty>,
    pub span:   Span,
}

#[derive(Debug)]
pub struct WhereTy {
    pub ty:    Box<Ty>,
    pub input: Box<Ty>,
    pub kind:  Kind,
    pub span:  Span,
}

#[derive(Debug)]
pub struct ErrorTy {
    pub span: Span,
}

impl Ty {
    pub fn span(&self) -> Span {
        match self {
            Ty::Var(ty) => ty.span,
            Ty::Lam(ty) => ty.span,
            Ty::App(ty) => ty.span,
            Ty::Where(ty) => ty.span,
            Ty::Error(ty) => ty.span,
        }
    }
}
