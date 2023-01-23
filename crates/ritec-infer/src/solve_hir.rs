use ritec_core::{Literal, UnaryOp};
use ritec_error::Diagnostic;
use ritec_hir as hir;

use crate::{
    As, InferType, Instance, ItemId, Projection, Solver, TypeProjection, TypeVariableKind,
};

impl<'a> Solver<'a> {
    pub fn solve_body(&mut self, body: &hir::Body) -> Result<(), Diagnostic> {
        for local in body.locals.values() {
            let ty = self.table_mut().infer_hir(&local.ty, &Instance::empty());
            self.table_mut().register_type(local.id, ty);
        }

        for block in body.blocks.values() {
            self.solve_block(body, block)?;
        }

        Ok(())
    }

    pub fn solve_block(&mut self, body: &hir::Body, block: &hir::Block) -> Result<(), Diagnostic> {
        for stmt in block.stmts.iter() {
            self.solve_stmt(body, stmt)?;
        }

        Ok(())
    }

    pub fn solve_stmt(&mut self, body: &hir::Body, stmt: &hir::Stmt) -> Result<(), Diagnostic> {
        match stmt {
            hir::Stmt::Let(stmt) => self.solve_let_stmt(body, stmt),
            hir::Stmt::Expr(stmt) => self.solve_expr_stmt(body, stmt),
        }
    }

    fn solve_let_stmt(&mut self, body: &hir::Body, stmt: &hir::LetStmt) -> Result<(), Diagnostic> {
        let local = &body.locals[stmt.local];
        let ty = self.register_type(local.id, &local.ty);

        if let Some(init) = stmt.init {
            let init_ty = self.solve_expr(body, &body.exprs[init])?;
            self.unify(ty, init_ty)?;
        }

        Ok(())
    }

    fn solve_expr_stmt(
        &mut self,
        body: &hir::Body,
        stmt: &hir::ExprStmt,
    ) -> Result<(), Diagnostic> {
        self.solve_expr(body, &body.exprs[stmt.expr])?;

        Ok(())
    }

    pub fn solve_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::Expr,
    ) -> Result<InferType, Diagnostic> {
        let ty = match expr {
            hir::Expr::Local(expr) => self.solve_local_expr(body, expr)?,
            hir::Expr::Literal(expr) => self.solve_literal_expr(body, expr)?,
            hir::Expr::Function(expr) => self.solve_function_expr(body, expr)?,
            hir::Expr::ClassInit(expr) => self.solve_init_expr(body, expr)?,
            hir::Expr::Field(expr) => self.solve_field_expr(body, expr)?,
            hir::Expr::As(expr) => self.solve_as_expr(body, expr)?,
            hir::Expr::Bitcast(expr) => self.solve_bitcast_expr(body, expr)?,
            hir::Expr::Sizeof(expr) => self.solve_sizeof_expr(body, expr)?,
            hir::Expr::Alignof(expr) => self.solve_alignof_expr(body, expr)?,
            hir::Expr::Malloc(expr) => self.solve_malloc_expr(body, expr)?,
            hir::Expr::Free(expr) => self.solve_free_expr(body, expr)?,
            hir::Expr::Memcpy(expr) => self.solve_memcpy_expr(body, expr)?,
            hir::Expr::Call(expr) => self.solve_call_expr(body, expr)?,
            hir::Expr::MethodCall(expr) => self.solve_method_call_expr(body, expr)?,
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
    ) -> Result<InferType, Diagnostic> {
        let local = &body.locals[expr.local];
        let ty = self.register_type(local.id, &local.ty);

        Ok(ty)
    }

    pub fn solve_literal_expr(
        &mut self,
        _body: &hir::Body,
        expr: &hir::LiteralExpr,
    ) -> Result<InferType, Diagnostic> {
        match expr.literal {
            Literal::Null(_) => {
                let var = self.table_mut().new_variable(None);
                Ok(InferType::apply(
                    ItemId::Pointer,
                    [InferType::Var(var)],
                    expr.span,
                ))
            }
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
    ) -> Result<InferType, Diagnostic> {
        let mut generics = Vec::new();
        for generic in expr.instance.generics.iter() {
            let ty = self.table_mut().infer_hir(generic, &Instance::empty());
            self.table_mut().register_generic(expr.id, ty.clone());
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
        expr: &hir::ClassInitExpr,
    ) -> Result<InferType, Diagnostic> {
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
    ) -> Result<InferType, Diagnostic> {
        let class = self.solve_expr(body, &body[expr.class])?;
        let proj = TypeProjection {
            base: Box::new(class),
            proj: Projection::Field(body[expr.class].id(), expr.field.clone()),
        };

        Ok(InferType::Proj(proj))
    }

    pub fn solve_as_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::AsExpr,
    ) -> Result<InferType, Diagnostic> {
        let ty = self.table_mut().infer_hir(&expr.ty, &Instance::empty());
        self.table_mut().register_generic(expr.id, ty.clone());

        let expr = self.solve_expr(body, &body[expr.expr])?;
        self.solve(As::new(expr, ty.clone()))?;

        Ok(ty)
    }

    pub fn solve_bitcast_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::BitcastExpr,
    ) -> Result<InferType, Diagnostic> {
        self.solve_expr(body, &body.exprs[expr.expr])?;
        Ok(self.table_mut().infer_hir(&expr.ty, &Instance::empty()))
    }

    pub fn solve_sizeof_expr(
        &mut self,
        _body: &hir::Body,
        expr: &hir::SizeofExpr,
    ) -> Result<InferType, Diagnostic> {
        let ty = self.table_mut().infer_hir(&expr.ty, &Instance::empty());
        self.table_mut().register_generic(expr.id, ty.clone());
        Ok(InferType::USIZE)
    }

    pub fn solve_alignof_expr(
        &mut self,
        _body: &hir::Body,
        expr: &hir::AlignofExpr,
    ) -> Result<InferType, Diagnostic> {
        let ty = self.table_mut().infer_hir(&expr.ty, &Instance::empty());
        self.table_mut().register_generic(expr.id, ty.clone());
        Ok(InferType::USIZE)
    }

    pub fn solve_malloc_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::MallocExpr,
    ) -> Result<InferType, Diagnostic> {
        let count = self.solve_expr(body, &body.exprs[expr.count])?;
        self.unify(count, InferType::USIZE)?;

        let ty = self.table_mut().infer_hir(&expr.ty, &Instance::empty());
        self.table_mut().register_generic(expr.id, ty.clone());

        Ok(InferType::apply(ItemId::Pointer, [ty], expr.span))
    }

    pub fn solve_free_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::FreeExpr,
    ) -> Result<InferType, Diagnostic> {
        let ptr = self.solve_expr(body, &body.exprs[expr.expr])?;

        let var = InferType::Var(self.table_mut().new_variable(None));
        self.unify(ptr, InferType::apply(ItemId::Pointer, [var], expr.span))?;

        Ok(InferType::void(expr.span))
    }

    pub fn solve_memcpy_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::MemcpyExpr,
    ) -> Result<InferType, Diagnostic> {
        let dst = self.solve_expr(body, &body.exprs[expr.dst])?;
        let src = self.solve_expr(body, &body.exprs[expr.src])?;
        let count = self.solve_expr(body, &body.exprs[expr.size])?;

        let var = InferType::Var(self.table_mut().new_variable(None));
        self.unify(dst.clone(), src)?;
        self.unify(dst, InferType::apply(ItemId::Pointer, [var], expr.span))?;

        self.unify(count, InferType::USIZE)?;

        Ok(InferType::void(expr.span))
    }

    pub fn solve_call_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::CallExpr,
    ) -> Result<InferType, Diagnostic> {
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

    pub fn solve_method_call_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::MethodCallExpr,
    ) -> Result<InferType, Diagnostic> {
        let mut generics = Vec::new();
        for generic in expr.generics.iter() {
            generics.push(self.table_mut().infer_hir(generic, &Instance::empty()));
        }

        let class = self.solve_expr(body, &body.exprs[expr.callee])?;
        let proj = TypeProjection {
            base: Box::new(class),
            proj: Projection::Method(body[expr.callee].id(), expr.method.clone(), generics),
        };

        let function = InferType::Proj(proj);

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
    ) -> Result<InferType, Diagnostic> {
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
    ) -> Result<InferType, Diagnostic> {
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
    ) -> Result<InferType, Diagnostic> {
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
    ) -> Result<InferType, Diagnostic> {
        assert_eq!(expr.operator, UnaryOp::Neg);

        self.solve_expr(body, &body.exprs[expr.operand])
    }

    pub fn solve_not_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::UnaryExpr,
    ) -> Result<InferType, Diagnostic> {
        assert_eq!(expr.operator, UnaryOp::Not);

        let ty = self.solve_expr(body, &body.exprs[expr.operand])?;
        self.unify(ty, InferType::apply(ItemId::Bool, vec![], expr.span))?;

        Ok(InferType::apply(ItemId::Bool, vec![], expr.span))
    }

    pub fn solve_binary_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::BinaryExpr,
    ) -> Result<InferType, Diagnostic> {
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
    ) -> Result<InferType, Diagnostic> {
        let lhs = self.solve_expr(body, &body.exprs[expr.lhs])?;
        let rhs = self.solve_expr(body, &body.exprs[expr.rhs])?;
        self.unify(lhs.clone(), rhs)?;

        Ok(lhs)
    }

    pub fn solve_return_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::ReturnExpr,
    ) -> Result<InferType, Diagnostic> {
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
    ) -> Result<InferType, Diagnostic> {
        Ok(InferType::void(expr.span))
    }

    pub fn solve_block_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::BlockExpr,
    ) -> Result<InferType, Diagnostic> {
        self.solve_block(body, &body[expr.block])?;
        Ok(InferType::void(expr.span))
    }

    pub fn solve_if_expr(
        &mut self,
        body: &hir::Body,
        expr: &hir::IfExpr,
    ) -> Result<InferType, Diagnostic> {
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
    ) -> Result<InferType, Diagnostic> {
        self.solve_block(body, &body[expr.block])?;
        Ok(InferType::void(expr.span))
    }
}
