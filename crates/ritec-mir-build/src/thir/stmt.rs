use ritec_core::Span;
use ritec_mir::LocalId;

use super::ExprId;

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Expr(ExprStmt),
    Let(LetStmt),
}

impl Stmt {
    pub const fn span(&self) -> Span {
        match self {
            Self::Let(stmt) => stmt.span,
            Self::Expr(stmt) => stmt.span,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LetStmt {
    pub local: LocalId,
    pub init: Option<ExprId>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExprStmt {
    pub expr: ExprId,
    pub span: Span,
}
