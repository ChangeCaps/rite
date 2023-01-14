use ritec_hir as hir;

use crate::{Error, InferType, ItemId, Solver};

impl Solver {
    pub fn solve_body(&mut self, body: &hir::Body) -> Result<(), Error> {
        for local in body.locals.values() {
            let ty = self.table_mut().infer_hir(&local.ty);
            self.table_mut().register_type(local.id, ty);
        }

        for stmt in body.stmts.values() {
            self.solve_stmt(body, stmt)?;
        }

        Ok(())
    }

    pub fn solve_stmt(&mut self, body: &hir::Body, stmt: &hir::Stmt) -> Result<(), Error> {
        match stmt {
            hir::Stmt::Let(stmt) => self.solve_let_stmt(body, stmt),
            hir::Stmt::Expr(stmt) => self.solve_expr_stmt(body, stmt),
        }
    }

    fn solve_let_stmt(&mut self, body: &hir::Body, stmt: &hir::LetStmt) -> Result<(), Error> {
        let local = &body.locals[stmt.local];
        let ty = self.register_type(local.id, &local.ty);

        if let Some(init) = stmt.init {
            let init_ty = self.solve_expr(body, &body.exprs[init])?;
            self.unify(ty, init_ty)?;
        }

        Ok(())
    }

    fn solve_expr_stmt(&mut self, body: &hir::Body, stmt: &hir::ExprStmt) -> Result<(), Error> {
        self.solve_expr(body, &body.exprs[stmt.expr])?;

        Ok(())
    }

    pub fn solve_expr(&mut self, body: &hir::Body, expr: &hir::Expr) -> Result<InferType, Error> {
        match expr {
            hir::Expr::Local(expr) => self.solve_local_expr(body, expr),
            hir::Expr::Ref(expr) => self.solve_ref_expr(body, expr),
            hir::Expr::Deref(expr) => self.solve_deref_expr(body, expr),
            hir::Expr::Assign(expr) => self.solve_assign_expr(body, expr),
            hir::Expr::Return(expr) => self.solve_return_expr(body, expr),
        }
    }

    pub fn solve_local_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::LocalExpr,
    ) -> Result<InferType, Error> {
        let local = &body.locals[expr.local];
        let ty = self.register_type(local.id, &local.ty);

        Ok(ty)
    }

    pub fn solve_ref_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::RefExpr,
    ) -> Result<InferType, Error> {
        let ty = self.solve_expr(body, &body.exprs[expr.operand])?;
        let pointer_type = InferType::apply(ItemId::Pointer, vec![ty], expr.span);
        (self.table_mut()).register_type(expr.id, pointer_type.clone());

        Ok(pointer_type)
    }

    /// Create a new variable `U` and unify `T = *U`, and return `U`.
    pub fn solve_deref_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::DerefExpr,
    ) -> Result<InferType, Error> {
        let pointer = self.solve_expr(body, &body.exprs[expr.operand])?;
        let pointee = InferType::from(self.new_variable());
        self.table_mut().register_type(expr.id, pointee.clone());

        self.unify(
            pointer,
            InferType::apply(ItemId::Pointer, vec![pointee.clone()], expr.span),
        )?;

        Ok(pointee)
    }

    pub fn solve_assign_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::AssignExpr,
    ) -> Result<InferType, Error> {
        let lhs = self.solve_expr(body, &body.exprs[expr.lhs])?;
        let rhs = self.solve_expr(body, &body.exprs[expr.rhs])?;

        self.table_mut().register_type(expr.id, lhs.clone());
        self.unify(lhs.clone(), rhs)?;

        Ok(lhs)
    }

    pub fn solve_return_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::ReturnExpr,
    ) -> Result<InferType, Error> {
        let ty = if let Some(value) = expr.value {
            self.solve_expr(body, &body.exprs[value])?
        } else {
            InferType::void(expr.span)
        };

        self.table_mut().register_type(expr.id, ty.clone());
        self.unify(ty.clone(), self.return_type().clone())?;
        Ok(InferType::void(expr.span))
    }
}
