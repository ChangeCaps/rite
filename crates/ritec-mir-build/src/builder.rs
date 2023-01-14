use ritec_mir as mir;

use crate::thir;

pub struct Builder<'a> {
    pub thir: &'a thir::Body,
    pub mir: mir::Body,
    pub current_block: mir::BlockId,
}

impl<'a> Builder<'a> {
    pub fn new(thir: &'a thir::Body) -> Self {
        let mut mir = mir::Body::new();
        let current_block = mir.blocks.push(mir::Block::new());

        Self {
            thir,
            mir,
            current_block,
        }
    }

    pub fn build(mut self) -> mir::Body {
        self.mir.locals = self.thir.locals.clone();

        for stmt in self.thir.stmts.values() {
            self.build_stmt(stmt);
        }

        self.mir.clone()
    }

    pub fn block(&self) -> &mir::Block {
        &self.mir.blocks[self.current_block]
    }

    pub fn block_mut(&mut self) -> &mut mir::Block {
        if self.block().is_terminated() {
            self.push_block();
        }

        &mut self.mir.blocks[self.current_block]
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
        self.current_block = block;
    }

    pub fn terminate(&mut self, term: mir::Terminator) -> mir::BlockId {
        self.block_mut().terminator = Some(term);
        self.current_block
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
}
