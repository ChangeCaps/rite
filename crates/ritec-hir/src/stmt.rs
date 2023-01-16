use ritec_core::Span;

use crate::{ExprId, HirId, LocalId};

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Let(LetStmt),
    Expr(ExprStmt),
}

impl Stmt {
    pub const fn span(&self) -> Span {
        match self {
            Self::Let(stmt) => stmt.span,
            Self::Expr(stmt) => stmt.span,
        }
    }

    pub const fn id(&self) -> HirId {
        match self {
            Self::Let(stmt) => stmt.id,
            Self::Expr(stmt) => stmt.id,
        }
    }
}

impl From<LetStmt> for Stmt {
    fn from(stmt: LetStmt) -> Self {
        Self::Let(stmt)
    }
}

impl From<ExprStmt> for Stmt {
    fn from(stmt: ExprStmt) -> Self {
        Self::Expr(stmt)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LetStmt {
    pub local: LocalId,
    pub init: Option<ExprId>,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExprStmt {
    pub expr: ExprId,
    pub id: HirId,
    pub span: Span,
}
