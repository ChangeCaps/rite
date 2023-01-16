use ritec_mir as mir;

use crate::{thir, unpack, FunctionBuilder};

impl<'a> FunctionBuilder<'a> {
    pub fn build_stmt(&mut self, block: mir::BlockId, stmt: &thir::Stmt) -> mir::BlockId {
        match stmt {
            thir::Stmt::Let(stmt) => self.build_let_stmt(block, stmt),
            thir::Stmt::Expr(stmt) => self.build_expr_stmt(block, stmt),
        }
    }

    pub fn build_let_stmt(
        &mut self,
        mut block: mir::BlockId,
        stmt: &thir::LetStmt,
    ) -> mir::BlockId {
        if let Some(init) = stmt.init {
            let init = unpack!(block = self.as_value(block, &self.thir.exprs[init]));
            self[block].push_assign(stmt.local, init);
        }

        block
    }

    pub fn build_expr_stmt(
        &mut self,
        mut block: mir::BlockId,
        stmt: &thir::ExprStmt,
    ) -> mir::BlockId {
        let expr = &self.thir[stmt.expr];
        let value = unpack!(block = self.as_value(block, expr));

        if !expr.ty().is_void() || matches!(expr, thir::Expr::Call(_)) {
            self[block].terminate_drop(value);
        }

        block
    }
}
