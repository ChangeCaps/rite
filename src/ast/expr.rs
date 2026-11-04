use crate::{ast::Phase, diagnostic::Span};

pub enum Expr<P>
where
    P: Phase,
{
    Item(ItemExpr<P>),
    Error(ErrorExpr<P>),
}

pub struct ItemExpr<P>
where
    P: Phase,
{
    pub name: P::Name,
    pub ty: P::Type,
    pub span: Span,
}

pub struct ErrorExpr<P>
where
    P: Phase,
{
    pub ty: P::Type,
    pub span: Span,
}
