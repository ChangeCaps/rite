use ritec_hir as hir;
use ritec_infer::{Error as InferError, InferenceTable};
use ritec_mir as mir;

use crate::thir;

#[derive(Clone, Debug, PartialEq)]
pub struct ThirBuilder<'a> {
    pub program: &'a hir::Program,
    pub hir: &'a hir::Body,
    pub thir: thir::Body,
    pub table: InferenceTable,
}

impl<'a> ThirBuilder<'a> {
    pub fn new(
        program: &'a hir::Program,
        hir: &'a hir::Body,
        table: InferenceTable,
    ) -> Result<Self, InferError> {
        Ok(Self {
            program,
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

    pub fn build_stmt(&mut self, stmt: &hir::Stmt) -> Result<thir::StmtId, InferError> {
        let stmt = match stmt {
            hir::Stmt::Let(stmt) => self.build_let_stmt(stmt)?,
            hir::Stmt::Expr(stmt) => self.build_expr_stmt(stmt)?,
        };

        Ok(self.thir.stmts.push(stmt))
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
            hir::Expr::Literal(expr) => self.build_literal_expr(expr)?,
            hir::Expr::Function(expr) => self.build_function_expr(expr)?,
            hir::Expr::Call(expr) => self.build_call_expr(expr)?,
            hir::Expr::Unary(expr) => self.build_unary_expr(expr)?,
            hir::Expr::Binary(expr) => self.build_binary_expr(expr)?,
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

    pub fn build_literal_expr(
        &mut self,
        expr: &hir::LiteralExpr,
    ) -> Result<thir::Expr, InferError> {
        Ok(thir::Expr::Literal(thir::LiteralExpr {
            literal: expr.literal.clone(),
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        }))
    }

    pub fn build_function_expr(
        &mut self,
        expr: &hir::FunctionExpr,
    ) -> Result<thir::Expr, InferError> {
        let mut generics = Vec::new();

        for i in 0..expr.instance.generics.len() {
            let ty = self.table.get_generic(expr.id, i).unwrap();
            generics.push(self.table.resolve_mir_type(ty)?);
        }

        let expr = thir::FunctionExpr {
            function: expr.instance.function,
            generics,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        };

        Ok(thir::Expr::Function(expr))
    }

    pub fn build_call_expr(&mut self, expr: &hir::CallExpr) -> Result<thir::Expr, InferError> {
        let callee = self.build_expr(&self.hir.exprs[expr.callee])?;

        let mut arguments = Vec::new();
        for &argument in expr.arguments.iter() {
            arguments.push(self.build_expr(&self.hir.exprs[argument])?);
        }

        let expr = thir::CallExpr {
            callee,
            arguments,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        };

        Ok(thir::Expr::Call(expr))
    }

    pub fn build_unary_expr(&mut self, expr: &hir::UnaryExpr) -> Result<thir::Expr, InferError> {
        let expr = thir::UnaryExpr {
            operator: expr.operator,
            operand: self.build_expr(&self.hir.exprs[expr.operand])?,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        };

        Ok(thir::Expr::Unary(expr))
    }

    pub fn build_binary_expr(&mut self, expr: &hir::BinaryExpr) -> Result<thir::Expr, InferError> {
        let expr = thir::BinaryExpr {
            operator: expr.operator,
            lhs: self.build_expr(&self.hir.exprs[expr.lhs])?,
            rhs: self.build_expr(&self.hir.exprs[expr.rhs])?,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        };

        Ok(thir::Expr::Binary(expr))
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
