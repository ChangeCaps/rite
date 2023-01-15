use crate::{thir, FunctionBuilder};

impl<'a> FunctionBuilder<'a> {
    pub fn build_stmt(&mut self, stmt: &thir::Stmt) {
        match stmt {
            thir::Stmt::Let(stmt) => self.build_let_stmt(stmt),
            thir::Stmt::Expr(stmt) => self.build_expr_stmt(stmt),
        }
    }

    pub fn build_let_stmt(&mut self, stmt: &thir::LetStmt) {
        if let Some(init) = stmt.init {
            let init = self.as_value(&self.thir.exprs[init]);
            self.push_assign(stmt.local, init);
        }
    }

    pub fn build_expr_stmt(&mut self, stmt: &thir::ExprStmt) {
        let value = self.as_value(&self.thir.exprs[stmt.expr]);

        if !matches!(self.thir[stmt.expr], thir::Expr::Return(_)) {
            self.push_drop(value);
        }
    }
}
