use std::{
    fmt::{self, Debug},
    ops::{Index, IndexMut},
};

use ritec_core::Arena;

use crate::{Expr, ExprId, Local, LocalId, Stmt};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HirId {
    pub index: usize,
}

impl HirId {
    pub const ZERO: Self = Self { index: 0 };

    pub fn increment(&mut self) -> Self {
        let id = *self;
        self.index += 1;
        id
    }
}

impl Debug for HirId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hir[{}]", self.index)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Body {
    pub locals: Arena<Local>,
    pub exprs: Arena<Expr>,
    pub stmts: Arena<Stmt>,
    pub next_id: HirId,
}

impl Body {
    pub const fn new() -> Self {
        Self {
            locals: Arena::new(),
            exprs: Arena::new(),
            stmts: Arena::new(),
            next_id: HirId::ZERO,
        }
    }

    pub fn next_id(&mut self) -> HirId {
        self.next_id.increment()
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
