mod let_stmt;

pub use let_stmt::*;

use ritec_span::Span;

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Let(LetStmt),
}

impl Stmt {
    pub const fn span(&self) -> Span {
        match self {
            Stmt::Let(stmt) => stmt.span,
        }
    }
}
