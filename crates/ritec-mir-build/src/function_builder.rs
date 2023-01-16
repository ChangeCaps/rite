use std::ops::{Index, IndexMut};

use ritec_mir as mir;

use crate::thir;

pub struct FunctionBuilder<'a> {
    pub thir: &'a thir::Body,
    pub mir: mir::Body,
    pub current_block: Option<mir::BlockId>,
    pub break_blocks: Vec<mir::BlockId>,
}

impl<'a> FunctionBuilder<'a> {
    pub fn new(thir: &'a thir::Body) -> Self {
        Self {
            thir,
            mir: mir::Body::new(),
            current_block: None,
            break_blocks: Vec::new(),
        }
    }

    pub fn build(mut self) -> mir::Body {
        self.mir.locals = self.thir.locals.clone();

        let entry_block = self.thir.blocks.values().next().unwrap();
        self.build_block(entry_block);

        if !self.block().is_terminated() {
            self.terminate(mir::Terminator::Return(mir::Operand::VOID));
        }

        self.mir.clone()
    }

    pub fn build_block(&mut self, block: &thir::Block) -> mir::BlockId {
        let block_id = self.push_block();

        for stmt in block.stmts.iter() {
            self.build_stmt(stmt);
        }

        block_id
    }

    pub fn current_block(&self) -> mir::BlockId {
        self.current_block.unwrap()
    }

    pub fn block(&self) -> &mir::Block {
        &self.mir.blocks[self.current_block()]
    }

    pub fn block_mut(&mut self) -> &mut mir::Block {
        if self.block().is_terminated() {
            self.push_block();
        }

        let block = self.current_block();
        &mut self.mir.blocks[block]
    }

    pub fn next_block(&self) -> mir::BlockId {
        self.mir.blocks.next_id()
    }

    pub fn reserve_block(&mut self) -> mir::BlockId {
        self.mir.blocks.push(mir::Block::new())
    }

    pub fn push_block(&mut self) -> mir::BlockId {
        let id = self.mir.blocks.push(mir::Block::new());
        self.set_block(id);
        id
    }

    pub fn set_block(&mut self, block: mir::BlockId) {
        self.current_block = Some(block);
    }

    pub fn terminate(&mut self, term: mir::Terminator) -> mir::BlockId {
        self.block_mut().terminator = Some(term);
        self.current_block.unwrap()
    }

    pub fn is_terminated(&self) -> bool {
        self.block().is_terminated()
    }

    pub fn push_statement(&mut self, stmt: impl Into<mir::Statement>) {
        self.block_mut().statements.push(stmt.into());
    }

    pub fn push_assign(&mut self, place: impl Into<mir::Place>, value: impl Into<mir::Value>) {
        let assign = mir::Assign {
            place: place.into(),
            value: value.into(),
        };

        self.push_statement(assign);
    }

    pub fn push_temp(&mut self, ty: mir::Type) -> mir::Place {
        let local = self.mir.locals.push(mir::Local { ident: None, ty });
        mir::Place {
            local: local.cast(),
            proj: vec![],
        }
    }

    pub fn push_drop(&mut self, place: impl Into<mir::Value>) {
        let drop = mir::Statement::Drop(place.into());
        self.push_statement(drop);
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
