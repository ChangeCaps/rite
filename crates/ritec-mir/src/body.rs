use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use ritec_core::Arena;

use crate::{Block, BlockId, Local, LocalId};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Body {
    pub locals: Arena<Local>,
    pub blocks: Arena<Block>,
}

impl Body {
    pub const fn new() -> Self {
        Self {
            locals: Arena::new(),
            blocks: Arena::new(),
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

impl Index<BlockId> for Body {
    type Output = Block;

    fn index(&self, index: BlockId) -> &Self::Output {
        &self.blocks[index]
    }
}

impl IndexMut<BlockId> for Body {
    fn index_mut(&mut self, index: BlockId) -> &mut Self::Output {
        &mut self.blocks[index]
    }
}

impl Display for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (id, local) in self.locals.iter() {
            writeln!(
                f,
                "\tlet _{}: {}; {}",
                id.as_raw_index(),
                local.ty,
                local.comment()
            )?;
        }

        for (id, block) in self.blocks.iter() {
            writeln!(f)?;
            write!(f, "\tbb{}: {}", id.as_raw_index(), block)?;
        }

        Ok(())
    }
}
