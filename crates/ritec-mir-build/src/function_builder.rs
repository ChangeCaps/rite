use std::ops::{Index, IndexMut};

use ritec_mir as mir;

use crate::thir;

pub struct BlockAnd<T> {
    pub block: mir::BlockId,
    pub value: T,
}

impl<T> BlockAnd<T> {
    pub const fn new(block: mir::BlockId, value: T) -> Self {
        Self { block, value }
    }
}

#[macro_export]
macro_rules! unpack {
    ($block:ident = $block_and:expr) => {{
        let $crate::BlockAnd { block, value } = $block_and;
        $block = block;
        value
    }};
}

pub struct FunctionBuilder<'a> {
    pub thir: &'a thir::Body,
    pub mir: mir::Body,
    pub break_block: Option<mir::BlockId>,
}

impl<'a> FunctionBuilder<'a> {
    pub fn new(thir: &'a thir::Body) -> Self {
        Self {
            thir,
            mir: mir::Body::new(),
            break_block: None,
        }
    }

    pub fn build(mut self) -> mir::Body {
        self.mir.locals = self.thir.locals.clone();

        let entry_block = self.thir.blocks.values().next().unwrap();
        let block_id = self.mir.blocks.push(mir::Block::new());
        self.build_block(block_id, entry_block);

        self.mir.clone()
    }

    pub fn build_block(&mut self, mut block_id: mir::BlockId, block: &thir::Block) -> mir::BlockId {
        if !self[block_id].is_empty() {
            block_id = self.mir.blocks.push(mir::Block::new());
        }

        for stmt in block.stmts.iter() {
            block_id = self.build_stmt(block_id, stmt);
        }

        block_id
    }

    pub fn new_block(&mut self) -> mir::BlockId {
        self.mir.blocks.push(mir::Block::new())
    }

    pub fn push_temp(&mut self, ty: mir::Type) -> mir::Place {
        let local = self.mir.locals.push(mir::Local { ident: None, ty });
        mir::Place {
            local: local.cast(),
            proj: vec![],
        }
    }
}

impl Index<mir::BlockId> for FunctionBuilder<'_> {
    type Output = mir::Block;

    fn index(&self, index: mir::BlockId) -> &Self::Output {
        &self.mir.blocks[index]
    }
}

impl IndexMut<mir::BlockId> for FunctionBuilder<'_> {
    fn index_mut(&mut self, index: mir::BlockId) -> &mut Self::Output {
        &mut self.mir.blocks[index]
    }
}