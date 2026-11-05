use crate::{
    ast::{Expr, Phase, Ty},
    diagnostic::Span,
};

pub enum Stmt<P>
where
    P: Phase,
{
    Assign(AssignStmt<P>),
    Ascribe(AscribeStmt<P>),
    Error(ErrorStmt),
}

pub struct AssignStmt<P>
where
    P: Phase,
{
    pub var:  P::Var,
    pub expr: Expr<P>,
    pub span: Span,
}

pub struct AscribeStmt<P>
where
    P: Phase,
{
    pub var:  P::Var,
    pub ty:   Ty<P>,
    pub span: Span,
}

pub struct ErrorStmt {
    pub span: Span,
}
