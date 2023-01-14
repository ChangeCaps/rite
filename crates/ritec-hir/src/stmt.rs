use ritec_core::{Id, Span};

use crate::{ExprId, LocalId, UniverseId};

pub type StmtId = Id<Stmt>;

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

    pub const fn id(&self) -> UniverseId {
        match self {
            Self::Let(stmt) => stmt.id,
            Self::Expr(stmt) => stmt.id,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LetStmt {
    pub local: LocalId,
    pub init: Option<ExprId>,
    pub id: UniverseId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExprStmt {
    pub expr: ExprId,
    pub id: UniverseId,
    pub span: Span,
}
