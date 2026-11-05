use crate::{ast::Phase, diagnostic::Span};

pub enum Expr<P>
where
    P: Phase,
{
    Let(LetExpr<P>),
    Var(VarExpr<P>),
    Lam(LamExpr<P>),
    App(AppExpr<P>),
    Error(ErrorExpr<P>),
}

pub struct LetExpr<P>
where
    P: Phase,
{
    pub var:    P::Var,
    pub input:  Box<Expr<P>>,
    pub output: Box<Expr<P>>,
    pub ty:     P::Type,
    pub span:   Span,
}

pub struct VarExpr<P>
where
    P: Phase,
{
    pub var:  P::Var,
    pub ty:   P::Type,
    pub span: Span,
}

pub struct LamExpr<P>
where
    P: Phase,
{
    pub var:    P::Var,
    pub output: Box<Expr<P>>,
    pub ty:     P::Type,
    pub span:   Span,
}

pub struct AppExpr<P>
where
    P: Phase,
{
    pub lambda: Box<Expr<P>>,
    pub input:  Box<Expr<P>>,
    pub ty:     P::Type,
    pub span:   Span,
}

pub struct ErrorExpr<P>
where
    P: Phase,
{
    pub ty:   P::Type,
    pub span: Span,
}

impl<P> Expr<P>
where
    P: Phase,
{
    pub fn span(&self) -> Span {
        match self {
            Self::Let(expr) => expr.span,
            Self::Var(expr) => expr.span,
            Self::Lam(expr) => expr.span,
            Self::App(expr) => expr.span,
            Self::Error(expr) => expr.span,
        }
    }
}
