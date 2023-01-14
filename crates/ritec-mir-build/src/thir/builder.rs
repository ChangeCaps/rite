use ritec_hir as hir;
use ritec_infer::{Error as InferError, InferenceTable};
use ritec_mir as mir;

use crate::thir;

#[derive(Clone, Debug, PartialEq)]
pub struct ThirBuilder<'a> {
    pub hir: &'a hir::Body,
    pub thir: thir::Body,
    pub table: InferenceTable,
}

impl<'a> ThirBuilder<'a> {
    pub fn new(hir: &'a hir::Body, table: InferenceTable) -> Result<Self, InferError> {
        Ok(Self {
            hir,
            thir: thir::Body::new(),
            table,
        })
    }

    pub fn build(&mut self) -> Result<thir::Body, InferError> {
        for (local_id, local) in self.hir.locals.iter() {
            let local = mir::Local {
                ident: Some(local.ident.clone()),
                ty: self.table.resolve_mir(local.id)?,
            };

            self.thir.locals.insert(local_id.cast(), local);
        }

        for stmt in self.hir.stmts.values() {
            self.build_stmt(stmt)?;
        }

        Ok(self.thir.clone())
    }

    pub fn build_stmt(&mut self, stmt: &hir::Stmt) -> Result<(), InferError> {
        let stmt = match stmt {
            hir::Stmt::Let(stmt) => self.build_let_stmt(stmt)?,
            hir::Stmt::Expr(stmt) => self.build_expr_stmt(stmt)?,
        };

        self.thir.stmts.push(stmt);

        Ok(())
    }

    pub fn build_let_stmt(&mut self, stmt: &hir::LetStmt) -> Result<thir::Stmt, InferError> {
        let init = if let Some(init) = stmt.init {
            Some(self.build_expr(&self.hir.exprs[init])?)
        } else {
            None
        };

        Ok(thir::Stmt::Let(thir::LetStmt {
            local: stmt.local.cast(),
            init,
            span: stmt.span,
        }))
    }

    pub fn build_expr_stmt(&mut self, stmt: &hir::ExprStmt) -> Result<thir::Stmt, InferError> {
        let expr = self.build_expr(&self.hir.exprs[stmt.expr])?;

        Ok(thir::Stmt::Expr(thir::ExprStmt {
            expr,
            span: stmt.span,
        }))
    }

    pub fn build_expr(&mut self, expr: &hir::Expr) -> Result<thir::ExprId, InferError> {
        let expr = match expr {
            hir::Expr::Local(expr) => self.build_local_expr(expr)?,
            hir::Expr::Ref(expr) => self.build_ref_expr(expr)?,
            hir::Expr::Deref(expr) => self.build_deref_expr(expr)?,
            hir::Expr::Assign(expr) => self.build_assign_expr(expr)?,
            hir::Expr::Return(expr) => self.build_return_expr(expr)?,
        };

        Ok(self.thir.exprs.push(expr))
    }

    pub fn build_local_expr(&mut self, expr: &hir::LocalExpr) -> Result<thir::Expr, InferError> {
        Ok(thir::Expr::Local(thir::LocalExpr {
            local: expr.local.cast(),
            ty: self.thir.locals[expr.local.cast()].ty.clone(),
            span: expr.span,
        }))
    }

    pub fn build_ref_expr(&mut self, expr: &hir::RefExpr) -> Result<thir::Expr, InferError> {
        let operand = self.build_expr(&self.hir.exprs[expr.operand])?;

        Ok(thir::Expr::Ref(thir::RefExpr {
            operand,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        }))
    }

    pub fn build_deref_expr(&mut self, expr: &hir::DerefExpr) -> Result<thir::Expr, InferError> {
        let operand = self.build_expr(&self.hir.exprs[expr.operand])?;

        Ok(thir::Expr::Deref(thir::DerefExpr {
            operand,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        }))
    }

    pub fn build_assign_expr(&mut self, expr: &hir::AssignExpr) -> Result<thir::Expr, InferError> {
        let lhs = self.build_expr(&self.hir.exprs[expr.lhs])?;
        let rhs = self.build_expr(&self.hir.exprs[expr.rhs])?;

        Ok(thir::Expr::Assign(thir::AssignExpr {
            lhs,
            rhs,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        }))
    }

    pub fn build_return_expr(&mut self, expr: &hir::ReturnExpr) -> Result<thir::Expr, InferError> {
        let value = if let Some(value) = expr.value {
            Some(self.build_expr(&self.hir.exprs[value])?)
        } else {
            None
        };

        Ok(thir::Expr::Return(thir::ReturnExpr {
            value,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        }))
    }
}
