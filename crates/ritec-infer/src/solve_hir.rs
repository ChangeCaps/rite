use ritec_core::{BinaryOp, Literal, UnaryOp};
use ritec_hir as hir;

use crate::{Error, InferType, ItemId, Solver, TypeVariableKind};

impl<'a> Solver<'a> {
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
        let ty = match expr {
            hir::Expr::Local(expr) => self.solve_local_expr(body, expr)?,
            hir::Expr::Literal(expr) => self.solve_literal_expr(body, expr)?,
            hir::Expr::Function(expr) => self.solve_function_expr(body, expr)?,
            hir::Expr::Call(expr) => self.solve_call_expr(body, expr)?,
            hir::Expr::Unary(expr) => self.solve_unary_expr(body, expr)?,
            hir::Expr::Binary(expr) => self.solve_binary_expr(body, expr)?,
            hir::Expr::Assign(expr) => self.solve_assign_expr(body, expr)?,
            hir::Expr::Return(expr) => self.solve_return_expr(body, expr)?,
        };

        self.table_mut().register_type(expr.id(), ty.clone());

        Ok(ty)
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

    pub fn solve_literal_expr(
        &mut self,
        _body: &hir::Body,
        expr: &hir::LiteralExpr,
    ) -> Result<InferType, Error> {
        match expr.literal {
            Literal::Bool(_) => Ok(InferType::apply(ItemId::Bool, vec![], expr.span)),
            Literal::Int(_) => {
                let var = (self.table_mut()).new_variable(Some(TypeVariableKind::Integer));
                Ok(InferType::Var(var))
            }
            Literal::Float(_) => {
                let var = self.table_mut().new_variable(Some(TypeVariableKind::Float));
                Ok(InferType::Var(var))
            }
        }
    }

    pub fn solve_function_expr(
        &mut self,
        _body: &hir::Body,
        expr: &hir::FunctionExpr,
    ) -> Result<InferType, Error> {
        for (i, generic) in expr.instance.generics.iter().enumerate() {
            let ty = self.table_mut().infer_hir(&generic);
            self.table_mut().register_generic(expr.id, i, ty);
        }

        Ok(self.register_type(expr.id, &hir::Type::Function(expr.ty.clone())))
    }

    pub fn solve_call_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::CallExpr,
    ) -> Result<InferType, Error> {
        let function = self.solve_expr(body, &body.exprs[expr.callee])?;

        let return_type = InferType::Var(self.table_mut().new_variable(None));

        let mut arguments = Vec::new();
        arguments.push(return_type.clone());
        for &argument in expr.arguments.iter() {
            let argument_ty = self.solve_expr(body, &body.exprs[argument])?;
            arguments.push(argument_ty);
        }

        self.unify(
            function,
            InferType::apply(ItemId::Function, arguments, expr.span),
        )?;

        Ok(return_type)
    }

    pub fn solve_unary_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::UnaryExpr,
    ) -> Result<InferType, Error> {
        match expr.operator {
            UnaryOp::Ref => self.solve_ref_expr(body, expr),
            UnaryOp::Deref => self.solve_deref_expr(body, expr),
        }
    }

    pub fn solve_ref_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::UnaryExpr,
    ) -> Result<InferType, Error> {
        assert_eq!(expr.operator, UnaryOp::Ref);

        let ty = self.solve_expr(body, &body.exprs[expr.operand])?;
        let pointer_type = InferType::apply(ItemId::Pointer, vec![ty], expr.span);

        Ok(pointer_type)
    }

    /// Create a new variable `U` and unify `T = *U`, and return `U`.
    pub fn solve_deref_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::UnaryExpr,
    ) -> Result<InferType, Error> {
        assert_eq!(expr.operator, UnaryOp::Deref);

        let pointer = self.solve_expr(body, &body.exprs[expr.operand])?;
        let pointee = InferType::from(self.new_variable());

        self.unify(
            pointer,
            InferType::apply(ItemId::Pointer, vec![pointee.clone()], expr.span),
        )?;

        Ok(pointee)
    }

    pub fn solve_binary_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::BinaryExpr,
    ) -> Result<InferType, Error> {
        let lhs = self.solve_expr(body, &body.exprs[expr.lhs])?;
        let rhs = self.solve_expr(body, &body.exprs[expr.rhs])?;
        self.unify(lhs.clone(), rhs.clone())?;

        let ty = match expr.operator {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => lhs,
        };

        Ok(ty)
    }

    pub fn solve_assign_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::AssignExpr,
    ) -> Result<InferType, Error> {
        let lhs = self.solve_expr(body, &body.exprs[expr.lhs])?;
        let rhs = self.solve_expr(body, &body.exprs[expr.rhs])?;
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

        self.unify(ty.clone(), self.return_type().clone())?;
        Ok(InferType::void(expr.span))
    }
}
