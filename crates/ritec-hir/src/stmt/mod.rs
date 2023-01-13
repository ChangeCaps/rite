mod let_stmt;

pub use let_stmt::*;

use ritec_core::{Id, Span};

use crate::UniverseId;

#[derive(Clone, Debug, PartialEq)]
pub enum StmtKind {
    Let(LetStmt),
}

impl StmtKind {
    pub const fn span(&self) -> Span {
        match self {
            Self::Let(stmt) => stmt.span,
        }
    }
}

pub type StmtId = Id<Stmt>;

#[derive(Clone, Debug, PartialEq)]
pub struct Stmt {
    pub id: UniverseId,
    pub kind: StmtKind,
}

impl Stmt {
    pub const fn spna(&self) -> Span {
        self.kind.span()
    }
}
