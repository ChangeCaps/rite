use std::ops::{Index, IndexMut};

use ritec_core::Arena;

use crate::{Expr, ExprId, Local, LocalId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UniverseId {
    pub index: usize,
}

impl UniverseId {
    pub fn increment(&mut self) -> Self {
        let id = *self;
        self.index += 1;
        id
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Body {
    pub local: Arena<Local>,
    pub exprs: Arena<Expr>,
    pub next_universe_id: UniverseId,
}

impl Body {
    pub fn next_universe_id(&mut self) -> UniverseId {
        self.next_universe_id.increment()
    }
}

impl Index<LocalId> for Body {
    type Output = Local;

    fn index(&self, index: LocalId) -> &Self::Output {
        &self.local[index]
    }
}

impl IndexMut<LocalId> for Body {
    fn index_mut(&mut self, index: LocalId) -> &mut Self::Output {
        &mut self.local[index]
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
