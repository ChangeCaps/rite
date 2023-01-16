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

        for stmt in self.hir.blocks.values() {
            self.build_block(stmt)?;
        }

        Ok(self.thir.clone())
    }

    pub fn build_block(&mut self, block: &hir::Block) -> Result<thir::BlockId, InferError> {
        let mut thir = thir::Block::new();

        let block_id = self.thir.blocks.reserve();

        for stmt in block.stmts.iter() {
            thir.push(self.build_stmt(stmt)?);
        }

        self.thir.blocks.insert(block_id, thir);
        Ok(block_id)
    }

    pub fn build_stmt(&mut self, stmt: &hir::Stmt) -> Result<thir::Stmt, InferError> {
        match stmt {
            hir::Stmt::Let(stmt) => self.build_let_stmt(stmt),
            hir::Stmt::Expr(stmt) => self.build_expr_stmt(stmt),
        }
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
            hir::Expr::Bitcast(expr) => self.build_bitcast_expr(expr)?,
            hir::Expr::Call(expr) => self.build_call_expr(expr)?,
            hir::Expr::Unary(expr) => self.build_unary_expr(expr)?,
            hir::Expr::Binary(expr) => self.build_binary_expr(expr)?,
            hir::Expr::Assign(expr) => self.build_assign_expr(expr)?,
            hir::Expr::Return(expr) => self.build_return_expr(expr)?,
            hir::Expr::Break(expr) => self.build_break_expr(expr)?,
            hir::Expr::Block(expr) => self.build_block_expr(expr)?,
            hir::Expr::If(expr) => self.build_if_expr(expr)?,
            hir::Expr::Loop(expr) => self.build_loop_expr(expr)?,
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

    pub fn build_bitcast_expr(
        &mut self,
        expr: &hir::BitcastExpr,
    ) -> Result<thir::Expr, InferError> {
        let expr = thir::BitcastExpr {
            expr: self.build_expr(&self.hir.exprs[expr.expr])?,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        };

        Ok(thir::Expr::Bitcast(expr))
    }

    pub fn build_call_expr(&mut self, expr: &hir::CallExpr) -> Result<thir::Expr, InferError> {
        let callee = self.build_expr(&self.hir[expr.callee])?;

        let mut arguments = Vec::new();
        for &argument in expr.arguments.iter() {
            arguments.push(self.build_expr(&self.hir[argument])?);
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
            operand: self.build_expr(&self.hir[expr.operand])?,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        };

        Ok(thir::Expr::Unary(expr))
    }

    pub fn build_binary_expr(&mut self, expr: &hir::BinaryExpr) -> Result<thir::Expr, InferError> {
        let expr = thir::BinaryExpr {
            operator: expr.operator,
            lhs: self.build_expr(&self.hir[expr.lhs])?,
            rhs: self.build_expr(&self.hir[expr.rhs])?,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        };

        Ok(thir::Expr::Binary(expr))
    }

    pub fn build_assign_expr(&mut self, expr: &hir::AssignExpr) -> Result<thir::Expr, InferError> {
        let lhs = self.build_expr(&self.hir[expr.lhs])?;
        let rhs = self.build_expr(&self.hir[expr.rhs])?;

        Ok(thir::Expr::Assign(thir::AssignExpr {
            lhs,
            rhs,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        }))
    }

    pub fn build_return_expr(&mut self, expr: &hir::ReturnExpr) -> Result<thir::Expr, InferError> {
        let value = if let Some(value) = expr.value {
            Some(self.build_expr(&self.hir[value])?)
        } else {
            None
        };

        Ok(thir::Expr::Return(thir::ReturnExpr {
            value,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        }))
    }

    pub fn build_break_expr(&mut self, expr: &hir::BreakExpr) -> Result<thir::Expr, InferError> {
        Ok(thir::Expr::Break(thir::BreakExpr {
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        }))
    }

    pub fn build_block_expr(&mut self, expr: &hir::BlockExpr) -> Result<thir::Expr, InferError> {
        let expr = thir::BlockExpr {
            block: self.build_block(&self.hir[expr.block])?,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        };

        Ok(thir::Expr::Block(expr))
    }

    pub fn build_if_expr(&mut self, expr: &hir::IfExpr) -> Result<thir::Expr, InferError> {
        let condition = self.build_expr(&self.hir[expr.condition])?;
        let then_block = self.build_expr(&self.hir[expr.then_expr])?;
        let else_block = if let Some(else_block) = expr.else_expr {
            Some(self.build_expr(&self.hir[else_block])?)
        } else {
            None
        };

        Ok(thir::Expr::If(thir::IfExpr {
            condition,
            then_expr: then_block,
            else_expr: else_block,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        }))
    }

    pub fn build_loop_expr(&mut self, expr: &hir::LoopExpr) -> Result<thir::Expr, InferError> {
        let block = self.build_block(&self.hir[expr.block])?;

        Ok(thir::Expr::Loop(thir::LoopExpr {
            block,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        }))
    }
}
