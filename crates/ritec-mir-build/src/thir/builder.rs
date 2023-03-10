use ritec_core::UnaryOp;
use ritec_error::Diagnostic;
use ritec_hir as hir;
use ritec_infer::{InferenceTable, Modification, Modifications};
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
    ) -> Result<Self, Diagnostic> {
        Ok(Self {
            program,
            hir,
            thir: thir::Body::new(),
            table,
        })
    }

    pub fn build(&mut self) -> Result<thir::Body, Diagnostic> {
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

    pub fn build_block(&mut self, block: &hir::Block) -> Result<thir::BlockId, Diagnostic> {
        let mut thir = thir::Block::new();

        let block_id = self.thir.blocks.reserve();

        for stmt in block.stmts.iter() {
            thir.push(self.build_stmt(stmt)?);
        }

        self.thir.blocks.insert(block_id, thir);
        Ok(block_id)
    }

    pub fn build_stmt(&mut self, stmt: &hir::Stmt) -> Result<thir::Stmt, Diagnostic> {
        match stmt {
            hir::Stmt::Let(stmt) => self.build_let_stmt(stmt),
            hir::Stmt::Expr(stmt) => self.build_expr_stmt(stmt),
        }
    }

    pub fn build_let_stmt(&mut self, stmt: &hir::LetStmt) -> Result<thir::Stmt, Diagnostic> {
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

    pub fn build_expr_stmt(&mut self, stmt: &hir::ExprStmt) -> Result<thir::Stmt, Diagnostic> {
        let expr = self.build_expr(&self.hir.exprs[stmt.expr])?;

        Ok(thir::Stmt::Expr(thir::ExprStmt {
            expr,
            span: stmt.span,
        }))
    }

    pub fn apply_ref(&mut self, expr: thir::Expr) -> thir::Expr {
        let ty = mir::Type::pointer(expr.ty().clone());
        let id = self.thir.exprs.push(expr);

        thir::Expr::Unary(thir::UnaryExpr {
            operator: UnaryOp::Ref,
            operand: id,
            ty,
            span: self.thir.exprs[id].span(),
        })
    }

    pub fn apply_deref(&mut self, expr: thir::Expr) -> thir::Expr {
        let mir::Type::Pointer(ty) = expr.ty().clone() else {
            unreachable!("expected pointer type");
        };

        let id = self.thir.exprs.push(expr);

        thir::Expr::Unary(thir::UnaryExpr {
            operator: UnaryOp::Deref,
            operand: id,
            span: self.thir.exprs[id].span(),
            ty: *ty.pointee,
        })
    }

    pub fn apply_modification(
        &mut self,
        expr: thir::Expr,
        modification: Modification,
    ) -> thir::Expr {
        match modification {
            Modification::Ref => self.apply_ref(expr),
            Modification::Deref => self.apply_deref(expr),
        }
    }

    pub fn apply_modifications(
        &mut self,
        mut expr: thir::Expr,
        modifications: &Modifications,
    ) -> thir::Expr {
        for modification in modifications.iter() {
            expr = self.apply_modification(expr, modification.clone());
        }

        expr
    }

    pub fn build_expr(&mut self, expr: &hir::Expr) -> Result<thir::ExprId, Diagnostic> {
        let hir_id = expr.id();

        let mut expr = match expr {
            hir::Expr::Local(expr) => self.build_local_expr(expr)?,
            hir::Expr::Literal(expr) => self.build_literal_expr(expr)?,
            hir::Expr::Function(expr) => self.build_function_expr(expr)?,
            hir::Expr::ClassInit(expr) => self.build_init_expr(expr)?,
            hir::Expr::Field(expr) => self.build_field_expr(expr)?,
            hir::Expr::As(expr) => self.build_as_expr(expr)?,
            hir::Expr::Bitcast(expr) => self.build_bitcast_expr(expr)?,
            hir::Expr::Sizeof(expr) => self.build_sizeof_expr(expr)?,
            hir::Expr::Alignof(expr) => self.build_alignof_expr(expr)?,
            hir::Expr::Malloc(expr) => self.build_malloc_expr(expr)?,
            hir::Expr::Free(expr) => self.build_free_expr(expr)?,
            hir::Expr::Memcpy(expr) => self.build_memcpy_expr(expr)?,
            hir::Expr::Call(expr) => self.build_call_expr(expr)?,
            hir::Expr::MethodCall(expr) => self.build_method_call_expr(expr)?,
            hir::Expr::Unary(expr) => self.build_unary_expr(expr)?,
            hir::Expr::Binary(expr) => self.build_binary_expr(expr)?,
            hir::Expr::Assign(expr) => self.build_assign_expr(expr)?,
            hir::Expr::Return(expr) => self.build_return_expr(expr)?,
            hir::Expr::Break(expr) => self.build_break_expr(expr)?,
            hir::Expr::Block(expr) => self.build_block_expr(expr)?,
            hir::Expr::If(expr) => self.build_if_expr(expr)?,
            hir::Expr::Loop(expr) => self.build_loop_expr(expr)?,
        };

        if let Some(modifications) = self.table.get_modifications(hir_id) {
            expr = self.apply_modifications(expr, &modifications.clone());
        }

        Ok(self.thir.exprs.push(expr))
    }

    pub fn build_local_expr(&mut self, expr: &hir::LocalExpr) -> Result<thir::Expr, Diagnostic> {
        Ok(thir::Expr::Local(thir::LocalExpr {
            local: expr.local.cast(),
            ty: self.thir.locals[expr.local.cast()].ty.clone(),
            span: expr.span,
        }))
    }

    pub fn build_literal_expr(
        &mut self,
        expr: &hir::LiteralExpr,
    ) -> Result<thir::Expr, Diagnostic> {
        Ok(thir::Expr::Literal(thir::LiteralExpr {
            literal: expr.literal.clone(),
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        }))
    }

    pub fn build_function_expr(
        &mut self,
        expr: &hir::FunctionExpr,
    ) -> Result<thir::Expr, Diagnostic> {
        let mut generics = Vec::new();

        for ty in self.table.get_generics(expr.id) {
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

    pub fn build_init_expr(&mut self, expr: &hir::ClassInitExpr) -> Result<thir::Expr, Diagnostic> {
        let ty = self.table.resolve_mir(expr.id)?;

        let mir::Type::Class(class) = ty.clone() else {
            unreachable!("init expr must be a class");
        };

        let mut fields = Vec::new();

        for (field, expr) in expr.fields.iter() {
            let expr = self.build_expr(&self.hir.exprs[*expr])?;
            fields.push((field.cast(), expr));
        }

        Ok(thir::Expr::ClassInit(thir::ClassInitExpr {
            class,
            fields,
            ty,
            span: expr.span,
        }))
    }

    pub fn build_field_expr(&mut self, expr: &hir::FieldExpr) -> Result<thir::Expr, Diagnostic> {
        let base = self.build_expr(&self.hir.exprs[expr.class])?;
        let ty = self.table.resolve_mir(expr.id)?;

        let field = self
            .table
            .get_field(self.hir[expr.class].id())
            .unwrap()
            .cast();

        Ok(thir::Expr::Field(thir::FieldExpr {
            class: base,
            field,
            ty,
            span: expr.span,
        }))
    }

    pub fn build_as_expr(&mut self, expr: &hir::AsExpr) -> Result<thir::Expr, Diagnostic> {
        let into = self.table.get_generics(expr.id)[0].clone();

        Ok(thir::Expr::As(thir::AsExpr {
            expr: self.build_expr(&self.hir.exprs[expr.expr])?,
            into: self.table.resolve_mir_type(&into)?,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        }))
    }

    pub fn build_bitcast_expr(
        &mut self,
        expr: &hir::BitcastExpr,
    ) -> Result<thir::Expr, Diagnostic> {
        let expr = thir::BitcastExpr {
            expr: self.build_expr(&self.hir.exprs[expr.expr])?,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        };

        Ok(thir::Expr::Bitcast(expr))
    }

    pub fn build_sizeof_expr(&mut self, expr: &hir::SizeofExpr) -> Result<thir::Expr, Diagnostic> {
        let item = self.table.get_generics(expr.id)[0].clone();

        Ok(thir::Expr::Sizeof(thir::SizeofExpr {
            item: self.table.resolve_mir_type(&item)?,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        }))
    }

    pub fn build_alignof_expr(
        &mut self,
        expr: &hir::AlignofExpr,
    ) -> Result<thir::Expr, Diagnostic> {
        let item = self.table.get_generics(expr.id)[0].clone();

        Ok(thir::Expr::Alignof(thir::AlignofExpr {
            item: self.table.resolve_mir_type(&item)?,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        }))
    }

    pub fn build_malloc_expr(&mut self, expr: &hir::MallocExpr) -> Result<thir::Expr, Diagnostic> {
        let item = self.table.get_generics(expr.id)[0].clone();

        Ok(thir::Expr::Malloc(thir::MallocExpr {
            item: self.table.resolve_mir_type(&item)?,
            count: self.build_expr(&self.hir.exprs[expr.count])?,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        }))
    }

    pub fn build_free_expr(&mut self, expr: &hir::FreeExpr) -> Result<thir::Expr, Diagnostic> {
        let expr = thir::FreeExpr {
            expr: self.build_expr(&self.hir.exprs[expr.expr])?,
            ty: mir::Type::Void,
            span: expr.span,
        };

        Ok(thir::Expr::Free(expr))
    }

    pub fn build_memcpy_expr(&mut self, expr: &hir::MemcpyExpr) -> Result<thir::Expr, Diagnostic> {
        let expr = thir::MemcpyExpr {
            dst: self.build_expr(&self.hir.exprs[expr.dst])?,
            src: self.build_expr(&self.hir.exprs[expr.src])?,
            size: self.build_expr(&self.hir.exprs[expr.size])?,
            ty: mir::Type::Void,
            span: expr.span,
        };

        Ok(thir::Expr::Memcpy(expr))
    }

    pub fn build_call_expr(&mut self, expr: &hir::CallExpr) -> Result<thir::Expr, Diagnostic> {
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

    pub fn build_method_call_expr(
        &mut self,
        expr: &hir::MethodCallExpr,
    ) -> Result<thir::Expr, Diagnostic> {
        let callee = self.build_expr(&self.hir[expr.callee])?;

        let mir::Type::Class(class_type) = self.thir[callee].ty().deref().clone() else {
            unreachable!("method call expr must be a class");
        };

        let class = &self.program.classes[class_type.class.cast()];
        let method = self.table.get_method(self.hir[expr.callee].id()).unwrap();
        let method = &class[method];

        let mut arguments = Vec::new();
        arguments.push(callee);

        for &argument in expr.arguments.iter() {
            arguments.push(self.build_expr(&self.hir[argument])?);
        }

        let mut generics = class_type.generics.clone();

        for ty in self.table.get_generics(self.hir[expr.callee].id()) {
            generics.push(self.table.resolve_mir_type(ty)?);
        }

        let expr = thir::StaticCallExpr {
            callee: method.function,
            generics,
            arguments,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        };

        Ok(thir::Expr::StaticCall(expr))
    }

    pub fn build_unary_expr(&mut self, expr: &hir::UnaryExpr) -> Result<thir::Expr, Diagnostic> {
        let expr = thir::UnaryExpr {
            operator: expr.operator,
            operand: self.build_expr(&self.hir[expr.operand])?,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        };

        Ok(thir::Expr::Unary(expr))
    }

    pub fn build_binary_expr(&mut self, expr: &hir::BinaryExpr) -> Result<thir::Expr, Diagnostic> {
        let expr = thir::BinaryExpr {
            operator: expr.operator,
            lhs: self.build_expr(&self.hir[expr.lhs])?,
            rhs: self.build_expr(&self.hir[expr.rhs])?,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        };

        Ok(thir::Expr::Binary(expr))
    }

    pub fn build_assign_expr(&mut self, expr: &hir::AssignExpr) -> Result<thir::Expr, Diagnostic> {
        let lhs = self.build_expr(&self.hir[expr.lhs])?;
        let rhs = self.build_expr(&self.hir[expr.rhs])?;

        Ok(thir::Expr::Assign(thir::AssignExpr {
            lhs,
            rhs,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        }))
    }

    pub fn build_return_expr(&mut self, expr: &hir::ReturnExpr) -> Result<thir::Expr, Diagnostic> {
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

    pub fn build_break_expr(&mut self, expr: &hir::BreakExpr) -> Result<thir::Expr, Diagnostic> {
        Ok(thir::Expr::Break(thir::BreakExpr {
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        }))
    }

    pub fn build_block_expr(&mut self, expr: &hir::BlockExpr) -> Result<thir::Expr, Diagnostic> {
        let expr = thir::BlockExpr {
            block: self.build_block(&self.hir[expr.block])?,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        };

        Ok(thir::Expr::Block(expr))
    }

    pub fn build_if_expr(&mut self, expr: &hir::IfExpr) -> Result<thir::Expr, Diagnostic> {
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

    pub fn build_loop_expr(&mut self, expr: &hir::LoopExpr) -> Result<thir::Expr, Diagnostic> {
        let block = self.build_block(&self.hir[expr.block])?;

        Ok(thir::Expr::Loop(thir::LoopExpr {
            block,
            ty: self.table.resolve_mir(expr.id)?,
            span: expr.span,
        }))
    }
}
