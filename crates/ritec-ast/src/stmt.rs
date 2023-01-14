use ritec_core::{Ident, Span};

use crate::{Expr, Type};

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Let(LetStmt),
    Expr(ExprStmt),
}

impl Stmt {
    pub const fn span(&self) -> Span {
        match self {
            Stmt::Let(stmt) => stmt.span,
            Stmt::Expr(expr) => expr.span,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExprStmt {
    pub expr: Expr,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LetStmt {
    pub ident: Ident,
    pub ty: Option<Type>,
    pub init: Option<Expr>,
    pub span: Span,
}
