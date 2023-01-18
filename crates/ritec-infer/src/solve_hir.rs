use ritec_core::{Literal, UnaryOp};
use ritec_hir as hir;

use crate::{
    Error, InferType, Instance, ItemId, Projection, Solver, TypeProjection, TypeVariableKind,
};

impl<'a> Solver<'a> {
    pub fn solve_body(&mut self, body: &hir::Body) -> Result<(), Error> {
        for local in body.locals.values() {
            let ty = self.table_mut().infer_hir(&local.ty, &Instance::empty());
            self.table_mut().register_type(local.id, ty);
        }

        for block in body.blocks.values() {
            self.solve_block(body, block)?;
        }

        Ok(())
    }

    pub fn solve_block(&mut self, body: &hir::Body, block: &hir::Block) -> Result<(), Error> {
        for stmt in block.stmts.iter() {
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
            hir::Expr::Init(expr) => self.solve_init_expr(body, expr)?,
            hir::Expr::Field(expr) => self.solve_field_expr(body, expr)?,
            hir::Expr::Bitcast(expr) => self.solve_bitcast_expr(body, expr)?,
            hir::Expr::Call(expr) => self.solve_call_expr(body, expr)?,
            hir::Expr::Unary(expr) => self.solve_unary_expr(body, expr)?,
            hir::Expr::Binary(expr) => self.solve_binary_expr(body, expr)?,
            hir::Expr::Assign(expr) => self.solve_assign_expr(body, expr)?,
            hir::Expr::Return(expr) => self.solve_return_expr(body, expr)?,
            hir::Expr::Break(expr) => self.solve_break_expr(body, expr)?,
            hir::Expr::Block(expr) => self.solve_block_expr(body, expr)?,
            hir::Expr::If(expr) => self.solve_if_expr(body, expr)?,
            hir::Expr::Loop(expr) => self.solve_loop_expr(body, expr)?,
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
        let mut generics = Vec::new();
        for (i, generic) in expr.instance.generics.iter().enumerate() {
            let ty = self.table_mut().infer_hir(generic, &Instance::empty());
            self.table_mut().register_generic(expr.id, i, ty.clone());
            generics.push(ty);
        }

        let function = self.program()[expr.instance.function].clone();
        let ty = hir::Type::Function(function.ty());
        let instance = Instance::new(function.generics.params.clone(), generics);

        Ok(self.table_mut().infer_hir(&ty, &instance))
    }

    pub fn solve_init_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::InitExpr,
    ) -> Result<InferType, Error> {
        let class = &self.program()[expr.class.class];

        let mut generics = Vec::new();
        for generic in expr.class.generics.iter() {
            let ty = self.table_mut().infer_hir(generic, &Instance::empty());
            generics.push(ty);
        }

        let instance = Instance::new(class.generics.params.clone(), generics);

        for (id, init) in expr.fields.iter() {
            let field = &class.fields[*id];
            let init_type = self.solve_expr(body, &body.exprs[*init])?;
            let field_type = self.table_mut().infer_hir(&field.ty, &instance);

            self.unify(field_type, init_type)?;
        }

        Ok(InferType::apply(
            ItemId::Class(expr.class.class.cast(), expr.class.ident.clone()),
            instance.types,
            expr.span,
        ))
    }

    pub fn solve_field_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::FieldExpr,
    ) -> Result<InferType, Error> {
        let class = self.solve_expr(body, &body.exprs[expr.class])?;
        let proj = TypeProjection {
            base: Box::new(class),
            proj: Projection::Field(expr.field.clone()),
        };

        Ok(InferType::Proj(proj))
    }

    pub fn solve_bitcast_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::BitcastExpr,
    ) -> Result<InferType, Error> {
        self.solve_expr(body, &body.exprs[expr.expr])?;
        Ok(self.table_mut().infer_hir(&expr.ty, &Instance::empty()))
    }

    pub fn solve_call_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::CallExpr,
    ) -> Result<InferType, Error> {
        let function = self.solve_expr(body, &body.exprs[expr.callee])?;

        let return_type = InferType::Var(self.table_mut().new_variable(None));

        let mut arguments = Vec::new();
        for &argument in expr.arguments.iter() {
            let argument_ty = self.solve_expr(body, &body.exprs[argument])?;
            arguments.push(argument_ty);
        }

        arguments.push(return_type.clone());

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
            UnaryOp::Neg => self.solve_neg_expr(body, expr),
            UnaryOp::Not => self.solve_not_expr(body, expr),
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

    pub fn solve_neg_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::UnaryExpr,
    ) -> Result<InferType, Error> {
        assert_eq!(expr.operator, UnaryOp::Neg);

        self.solve_expr(body, &body.exprs[expr.operand])
    }

    pub fn solve_not_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::UnaryExpr,
    ) -> Result<InferType, Error> {
        assert_eq!(expr.operator, UnaryOp::Not);

        let ty = self.solve_expr(body, &body.exprs[expr.operand])?;
        self.unify(ty, InferType::apply(ItemId::Bool, vec![], expr.span))?;

        Ok(InferType::apply(ItemId::Bool, vec![], expr.span))
    }

    pub fn solve_binary_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::BinaryExpr,
    ) -> Result<InferType, Error> {
        let lhs = self.solve_expr(body, &body.exprs[expr.lhs])?;
        let rhs = self.solve_expr(body, &body.exprs[expr.rhs])?;
        self.unify(lhs.clone(), rhs.clone())?;

        let ty = if expr.operator.is_comparison() {
            InferType::apply(ItemId::Bool, vec![], expr.span)
        } else {
            lhs
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

    pub fn solve_break_expr(
        &mut self,
        _body: &hir::Body,
        expr: &hir::BreakExpr,
    ) -> Result<InferType, Error> {
        Ok(InferType::void(expr.span))
    }

    pub fn solve_block_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::BlockExpr,
    ) -> Result<InferType, Error> {
        self.solve_block(body, &body[expr.block])?;
        Ok(InferType::void(expr.span))
    }

    pub fn solve_if_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::IfExpr,
    ) -> Result<InferType, Error> {
        let condition = self.solve_expr(body, &body.exprs[expr.condition])?;
        self.unify(condition, InferType::apply(ItemId::Bool, vec![], expr.span))?;

        self.solve_expr(body, &body[expr.then_expr])?;

        if let Some(else_block) = expr.else_expr {
            self.solve_expr(body, &body[else_block])?;
        }

        Ok(InferType::void(expr.span))
    }

    pub fn solve_loop_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::LoopExpr,
    ) -> Result<InferType, Error> {
        self.solve_block(body, &body[expr.block])?;
        Ok(InferType::void(expr.span))
    }
}
