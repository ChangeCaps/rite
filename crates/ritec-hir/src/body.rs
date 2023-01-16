use std::{
    fmt::{self, Debug},
    ops::{Index, IndexMut},
};

use ritec_core::{Arena, Ident, Span};

use crate::{
    BitcastExpr, Block, BlockId, Expr, ExprId, ExprStmt, Local, LocalExpr, LocalId, ReturnExpr,
    Stmt, Type,
};

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
    pub blocks: Arena<Block>,
    pub next_id: HirId,
}

impl Body {
    pub const fn new() -> Self {
        Self {
            locals: Arena::new(),
            exprs: Arena::new(),
            blocks: Arena::new(),
            next_id: HirId::ZERO,
        }
    }

    pub fn block(&mut self, block: Block) -> BlockId {
        self.blocks.push(block)
    }

    pub fn local(&mut self, ident: impl Into<Ident>, ty: impl Into<Type>) -> LocalId {
        let id = self.next_id();
        let local = Local {
            ident: ident.into(),
            ty: ty.into(),
            id,
        };
        self.locals.push(local)
    }

    pub fn local_expr(&mut self, local: LocalId) -> ExprId {
        let id = self.next_id();
        let expr = LocalExpr {
            local,
            id,
            span: Span::DUMMY,
        };
        self.exprs.push(Expr::Local(expr))
    }

    pub fn bitcast_expr(&mut self, expr: ExprId, ty: impl Into<Type>) -> ExprId {
        let id = self.next_id();
        let expr = BitcastExpr {
            expr,
            ty: ty.into(),
            id,
            span: Span::DUMMY,
        };
        self.exprs.push(Expr::Bitcast(expr))
    }

    pub fn return_expr(&mut self, value: Option<ExprId>) -> ExprId {
        let id = self.next_id();
        let expr = ReturnExpr {
            value,
            id,
            span: Span::DUMMY,
        };
        self.exprs.push(Expr::Return(expr))
    }

    pub fn expr_stmt(&mut self, expr: ExprId) {
        let stmt = Stmt::Expr(ExprStmt {
            expr,
            id: self.next_id(),
            span: Span::DUMMY,
        });
        self.push_stmt(stmt)
    }

    pub fn push_stmt(&mut self, stmt: Stmt) {
        if self.blocks.is_empty() {
            self.block(Block::new());
        }

        let block = self.blocks.last_mut().unwrap();
        block.push(stmt);
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
