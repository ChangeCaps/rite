use ritec_ast as ast;
use ritec_core::{Ident, UnaryOp};
use ritec_error::Diagnostic;
use ritec_hir as hir;

use crate::Resolver;

pub struct BodyLowerer<'a> {
    pub body: &'a mut hir::Body,
    pub resolver: Resolver<'a>,
    pub scope: Vec<hir::LocalId>,
}

impl<'a> BodyLowerer<'a> {
    pub fn new(body: &'a mut hir::Body, resolver: Resolver<'a>) -> Self {
        let scope = body.locals.keys().collect();

        Self {
            body,
            resolver,
            scope,
        }
    }

    pub fn lower_type(&self, ty: &ast::Type) -> Result<hir::Type, Diagnostic> {
        self.resolver.resolve_type(ty)
    }

    pub fn lower_block(&mut self, block: &ast::Block) -> Result<hir::BlockId, Diagnostic> {
        let mut hir = hir::Block::new();

        let block_id = self.body.blocks.reserve();
        let scope_index = self.scope.len();

        for stmt in block.stmts.iter() {
            hir.push(self.lower_stmt(stmt)?);
        }

        self.scope.truncate(scope_index);

        self.body.blocks.insert(block_id, hir);
        Ok(block_id)
    }

    pub fn lower_stmt(&mut self, stmt: &ast::Stmt) -> Result<hir::Stmt, Diagnostic> {
        match stmt {
            ast::Stmt::Let(stmt) => self.lower_let_stmt(stmt),
            ast::Stmt::Expr(stmt) => self.lower_expr_stmt(stmt),
        }
    }

    pub fn lower_let_stmt(&mut self, stmt: &ast::LetStmt) -> Result<hir::Stmt, Diagnostic> {
        let ty = if let Some(ty) = &stmt.ty {
            self.lower_type(ty)?
        } else {
            hir::Type::inferred(stmt.ident.span())
        };

        let local = hir::Local {
            ident: stmt.ident.clone(),
            ty,
            id: self.body.next_id(),
        };

        let local = self.body.locals.push(local);
        self.scope.push(local);

        let init = if let Some(ref init) = stmt.init {
            Some(self.lower_expr(init)?)
        } else {
            None
        };

        let let_stmt = hir::LetStmt {
            local,
            init,
            id: self.body.next_id(),
            span: stmt.span,
        };

        Ok(hir::Stmt::Let(let_stmt))
    }

    pub fn lower_expr_stmt(&mut self, stmt: &ast::ExprStmt) -> Result<hir::Stmt, Diagnostic> {
        let expr = self.lower_expr(&stmt.expr)?;

        let expr_stmt = hir::ExprStmt {
            expr,
            id: self.body.next_id(),
            span: stmt.span,
        };

        Ok(hir::Stmt::Expr(expr_stmt))
    }

    pub fn lower_expr(&mut self, expr: &ast::Expr) -> Result<hir::ExprId, Diagnostic> {
        let expr = match expr {
            ast::Expr::Paren(expr) => return self.lower_paren_expr(expr),
            ast::Expr::Path(expr) => self.lower_path_expr(expr)?,
            ast::Expr::Literal(expr) => self.lower_literal_expr(expr)?,
            ast::Expr::Init(expr) => self.lower_init_expr(expr)?,
            ast::Expr::Field(expr) => self.lower_field_expr(expr)?,
            ast::Expr::Call(expr) => self.lower_call_expr(expr)?,
            ast::Expr::Unary(expr) => self.lower_unary_expr(expr)?,
            ast::Expr::Binary(expr) => self.lower_binary_expr(expr)?,
            ast::Expr::Assign(expr) => self.lower_assign_expr(expr)?,
            ast::Expr::Return(expr) => self.lower_return_expr(expr)?,
            ast::Expr::Break(expr) => self.lower_break_expr(expr)?,
            ast::Expr::Block(expr) => self.lower_block_expr(expr)?,
            ast::Expr::If(expr) => self.lower_if_expr(expr)?,
            ast::Expr::Loop(expr) => self.lower_loop_expr(expr)?,
            ast::Expr::While(expr) => self.lower_while_expr(expr)?,
        };

        Ok(self.body.exprs.push(expr))
    }

    pub fn lower_paren_expr(&mut self, expr: &ast::ParenExpr) -> Result<hir::ExprId, Diagnostic> {
        self.lower_expr(&expr.expr)
    }

    pub fn find_local(&self, ident: &Ident) -> Option<hir::LocalId> {
        for &local_id in self.scope.iter().rev() {
            let local = &self.body[local_id];

            if local.ident == *ident {
                return Some(local_id);
            }
        }

        None
    }

    pub fn lower_path_expr(&mut self, expr: &ast::PathExpr) -> Result<hir::Expr, Diagnostic> {
        if let Some(ident) = expr.path.get_ident() {
            if let Some(local) = self.find_local(ident) {
                let local_expr = hir::LocalExpr {
                    local,
                    id: self.body.next_id(),
                    span: expr.span,
                };

                return Ok(hir::Expr::Local(local_expr));
            }
        }

        if expr.path.is_self() {
            if let Some(local) = self.find_local(&Ident::new("self", expr.span)) {
                let local_expr = hir::LocalExpr {
                    local,
                    id: self.body.next_id(),
                    span: expr.span,
                };

                return Ok(hir::Expr::Local(local_expr));
            }
        }

        if let Some(instance) = self.resolver.resolve_function(&expr.path)? {
            let function_expr = hir::FunctionExpr {
                instance,
                id: self.body.next_id(),
                span: expr.span,
            };

            return Ok(hir::Expr::Function(function_expr));
        }

        let err = Diagnostic::error(format!("'{}' not defined", expr.path))
            .with_msg_span("variable not found", expr.span);

        Err(err)
    }

    pub fn lower_literal_expr(&mut self, expr: &ast::LiteralExpr) -> Result<hir::Expr, Diagnostic> {
        let literal_expr = hir::LiteralExpr {
            literal: expr.literal.clone(),
            id: self.body.next_id(),
            span: expr.span,
        };

        Ok(hir::Expr::Literal(literal_expr))
    }

    pub fn lower_init_expr(&mut self, expr: &ast::InitExpr) -> Result<hir::Expr, Diagnostic> {
        let ty = self.resolver.resolve_path_type(&expr.class)?;

        let hir::Type::Class(class_type) = ty else {
            let err = Diagnostic::error(format!("'{}' is not a class", expr.class.path))
                .with_msg_span("expected class", expr.class.span);

            return Err(err);
        };

        let class = &self.resolver.program[class_type.class];

        let mut fields = Vec::new();
        for field in &expr.fields {
            let Some(field_id) = class.find_field(&field.ident) else {
                let err = Diagnostic::error(format!("'{}' has no field '{}'", class.ident, field.ident))
                    .with_msg_span("field not found", field.ident.span());

                return Err(err);
            };

            let field_init = self.lower_expr(&field.expr)?;

            fields.push((field_id, field_init));
        }

        let init_expr = hir::InitExpr {
            class: class_type,
            fields,
            id: self.body.next_id(),
            span: expr.span,
        };

        Ok(hir::Expr::Init(init_expr))
    }

    pub fn lower_field_expr(&mut self, expr: &ast::FieldExpr) -> Result<hir::Expr, Diagnostic> {
        let field_expr = hir::FieldExpr {
            class: self.lower_expr(&expr.class)?,
            field: expr.field.clone(),
            id: self.body.next_id(),
            span: expr.span,
        };

        Ok(hir::Expr::Field(field_expr))
    }

    pub fn lower_call_expr(&mut self, expr: &ast::CallExpr) -> Result<hir::Expr, Diagnostic> {
        let callee = self.lower_expr(&expr.callee)?;
        let mut arguments = Vec::new();

        for arg in expr.arguments.iter() {
            arguments.push(self.lower_expr(arg)?);
        }

        let call_expr = hir::CallExpr {
            callee,
            arguments,
            id: self.body.next_id(),
            span: expr.span,
        };

        Ok(hir::Expr::Call(call_expr))
    }

    pub fn lower_unary_expr(&mut self, expr: &ast::UnaryExpr) -> Result<hir::Expr, Diagnostic> {
        let unary_expr = hir::UnaryExpr {
            operator: expr.operator,
            operand: self.lower_expr(&expr.operand)?,
            id: self.body.next_id(),
            span: expr.span,
        };

        Ok(hir::Expr::Unary(unary_expr))
    }

    pub fn lower_binary_expr(&mut self, expr: &ast::BinaryExpr) -> Result<hir::Expr, Diagnostic> {
        let binary_expr = hir::BinaryExpr {
            operator: expr.operator,
            lhs: self.lower_expr(&expr.lhs)?,
            rhs: self.lower_expr(&expr.rhs)?,
            id: self.body.next_id(),
            span: expr.span,
        };

        Ok(hir::Expr::Binary(binary_expr))
    }

    pub fn lower_assign_expr(&mut self, expr: &ast::AssignExpr) -> Result<hir::Expr, Diagnostic> {
        let assign_expr = hir::AssignExpr {
            lhs: self.lower_expr(&expr.lhs)?,
            rhs: self.lower_expr(&expr.rhs)?,
            id: self.body.next_id(),
            span: expr.span,
        };

        Ok(hir::Expr::Assign(assign_expr))
    }

    pub fn lower_return_expr(&mut self, expr: &ast::ReturnExpr) -> Result<hir::Expr, Diagnostic> {
        let value = if let Some(expr) = &expr.value {
            Some(self.lower_expr(expr)?)
        } else {
            None
        };

        let return_expr = hir::ReturnExpr {
            value,
            id: self.body.next_id(),
            span: expr.span,
        };

        Ok(hir::Expr::Return(return_expr))
    }

    pub fn lower_break_expr(&mut self, expr: &ast::BreakExpr) -> Result<hir::Expr, Diagnostic> {
        let break_expr = hir::BreakExpr {
            id: self.body.next_id(),
            span: expr.span,
        };

        Ok(hir::Expr::Break(break_expr))
    }

    pub fn lower_block_expr(&mut self, expr: &ast::BlockExpr) -> Result<hir::Expr, Diagnostic> {
        let block_expr = hir::BlockExpr {
            block: self.lower_block(&expr.block)?,
            id: self.body.next_id(),
            span: expr.span,
        };

        Ok(hir::Expr::Block(block_expr))
    }

    pub fn lower_if_expr(&mut self, expr: &ast::IfExpr) -> Result<hir::Expr, Diagnostic> {
        let condition = self.lower_expr(&expr.condition)?;
        let then_expr = self.lower_expr(&expr.then_block)?;
        let else_expr = if let Some(else_block) = &expr.else_block {
            Some(self.lower_expr(else_block)?)
        } else {
            None
        };

        let if_expr = hir::IfExpr {
            condition,
            then_expr,
            else_expr,
            id: self.body.next_id(),
            span: expr.span,
        };

        Ok(hir::Expr::If(if_expr))
    }

    pub fn lower_loop_expr(&mut self, expr: &ast::LoopExpr) -> Result<hir::Expr, Diagnostic> {
        let loop_expr = hir::LoopExpr {
            block: self.lower_block(&expr.block)?,
            id: self.body.next_id(),
            span: expr.span,
        };

        Ok(hir::Expr::Loop(loop_expr))
    }

    pub fn lower_while_expr(&mut self, expr: &ast::WhileExpr) -> Result<hir::Expr, Diagnostic> {
        let mut block = expr.block.clone();

        let if_expr = ast::IfExpr {
            condition: Box::new(ast::Expr::Unary(ast::UnaryExpr {
                operator: UnaryOp::Not,
                operand: expr.condition.clone(),
                span: expr.condition.span(),
            })),
            then_block: Box::new(ast::Expr::Break(ast::BreakExpr { span: expr.span })),
            else_block: None,
            span: expr.span,
        };

        block.stmts.insert(
            0,
            ast::Stmt::Expr(ast::ExprStmt {
                expr: ast::Expr::If(if_expr),
                span: expr.condition.span(),
            }),
        );

        let loop_expr = ast::LoopExpr {
            block,
            span: expr.span,
        };

        self.lower_loop_expr(&loop_expr)
    }
}
