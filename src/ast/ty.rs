use crate::{ast::Phase, diagnostic::Span};

pub enum TypeLit<P>
where
    P: Phase,
{
    Ty(TyTy),
    Var(VarTy<P>),
    Lam(LamTy<P>),
    App(AppTy<P>),
    Error(ErrorTy),
}

pub struct TyTy {
    pub span: Span,
}

pub struct VarTy<P>
where
    P: Phase,
{
    pub var:  P::Var,
    pub span: Span,
}

pub struct LamTy<P>
where
    P: Phase,
{
    pub input:  Box<TypeLit<P>>,
    pub output: Box<TypeLit<P>>,
    pub span:   Span,
}

pub struct AppTy<P>
where
    P: Phase,
{
    pub lambda: Box<TypeLit<P>>,
    pub input:  Box<TypeLit<P>>,
    pub span:   Span,
}

pub struct ErrorTy {
    pub span: Span,
}

impl<P> TypeLit<P>
where
    P: Phase,
{
    pub fn span(&self) -> Span {
        match self {
            TypeLit::Ty(ty) => ty.span,
            TypeLit::Var(ty) => ty.span,
            TypeLit::Lam(ty) => ty.span,
            TypeLit::App(ty) => ty.span,
            TypeLit::Error(ty) => ty.span,
        }
    }
}
