use crate::{ast::Phase, diagnostic::Span};

pub enum Ty<P>
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
    pub input:  Box<Ty<P>>,
    pub output: Box<Ty<P>>,
    pub span:   Span,
}

pub struct AppTy<P>
where
    P: Phase,
{
    pub lambda: Box<Ty<P>>,
    pub input:  Box<Ty<P>>,
    pub span:   Span,
}

pub struct ErrorTy {
    pub span: Span,
}

impl<P> Ty<P>
where
    P: Phase,
{
    pub fn span(&self) -> Span {
        match self {
            Ty::Ty(ty) => ty.span,
            Ty::Var(ty) => ty.span,
            Ty::Lam(ty) => ty.span,
            Ty::App(ty) => ty.span,
            Ty::Error(ty) => ty.span,
        }
    }
}
