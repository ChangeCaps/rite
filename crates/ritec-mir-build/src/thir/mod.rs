mod builder;
mod expr;
mod stmt;

pub use builder::*;
pub use expr::*;
pub use stmt::*;

use std::ops::Index;

use ritec_core::Arena;
use ritec_mir::{Local, LocalId};

#[derive(Clone, Debug, PartialEq)]
pub struct Body {
    pub locals: Arena<Local>,
    pub exprs: Arena<Expr>,
    pub stmts: Arena<Stmt>,
}

impl Body {
    pub const fn new() -> Self {
        Self {
            locals: Arena::new(),
            exprs: Arena::new(),
            stmts: Arena::new(),
        }
    }
}

impl Index<LocalId> for Body {
    type Output = Local;

    fn index(&self, index: LocalId) -> &Self::Output {
        &self.locals[index]
    }
}

impl Index<ExprId> for Body {
    type Output = Expr;

    fn index(&self, index: ExprId) -> &Self::Output {
        &self.exprs[index]
    }
}

impl Index<StmtId> for Body {
    type Output = Stmt;

    fn index(&self, index: StmtId) -> &Self::Output {
        &self.stmts[index]
    }
}
