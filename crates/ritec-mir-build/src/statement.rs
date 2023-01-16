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
        let expr = &self.thir[stmt.expr];
        let value = self.as_value(expr);

        if !expr.ty().is_void() || matches!(expr, thir::Expr::Call(_)) {
            self.push_drop(value);
        }
    }
}
