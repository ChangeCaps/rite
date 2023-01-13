mod let_stmt;

pub use let_stmt::*;

use ritec_core::{Id, Span};

use crate::UniverseId;

pub type StmtId = Id<Stmt>;

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Let(LetStmt),
}

impl Stmt {
    pub const fn span(&self) -> Span {
        match self {
            Self::Let(stmt) => stmt.span,
        }
    }

    pub const fn id(&self) -> UniverseId {
        match self {
            Self::Let(stmt) => stmt.id,
        }
    }
}
