use std::ops::{Index, IndexMut};

use ritec_core::Arena;

use crate::{Expr, ExprId, Local, LocalId, Stmt, StmtId};

#[derive(Clone, Debug, Default, PartialEq)]
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

impl IndexMut<LocalId> for Body {
    fn index_mut(&mut self, index: LocalId) -> &mut Self::Output {
        &mut self.locals[index]
    }
}

impl Index<ExprId> for Body {
    type Output = Expr;

    fn index(&self, index: ExprId) -> &Self::Output {
        &self.exprs[index]
    }
}

impl IndexMut<ExprId> for Body {
    fn index_mut(&mut self, index: ExprId) -> &mut Self::Output {
        &mut self.exprs[index]
    }
}

impl Index<StmtId> for Body {
    type Output = Stmt;

    fn index(&self, index: StmtId) -> &Self::Output {
        &self.stmts[index]
    }
}

impl IndexMut<StmtId> for Body {
    fn index_mut(&mut self, index: StmtId) -> &mut Self::Output {
        &mut self.stmts[index]
    }
}
