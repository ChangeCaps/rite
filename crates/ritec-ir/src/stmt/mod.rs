mod block;
mod let_stmt;

pub use block::*;
pub use let_stmt::*;

use ritec_arena::Id;
use ritec_span::Span;

#[derive(Clone, Debug, PartialEq)]
pub enum StmtKind {
    Let(LetStmt),
    Block(BlockStmt),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

pub type StmtId = Id<Stmt>;

impl Stmt {
    pub const fn span(&self) -> Span {
        self.span
    }
}
