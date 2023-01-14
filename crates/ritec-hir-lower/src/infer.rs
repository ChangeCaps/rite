use ritec_error::Diagnostic;
use ritec_hir as hir;
use ritec_infer::{InferType, ItemId, Unify};
use ritec_mir as mir;

use crate::BodyLowerer;

impl<'a> BodyLowerer<'a> {
    pub fn infer_type(&mut self, ty: &hir::Type, id: hir::UniverseId) -> InferType {
        if let Some(ty) = self.types.get(&id) {
            return ty.clone();
        }

        let ty = self.solver.infer_type(ty);
        self.types.insert(id, ty.clone());
        ty
    }

    pub fn new_type_variable(&mut self, id: hir::UniverseId) -> InferType {
        if let Some(ty) = self.types.get(&id) {
            return ty.clone();
        }

        let var = self.solver.new_variable();
        let ty = InferType::Var(var);
        self.types.insert(id, ty.clone());
        ty
    }

    pub fn push_block(&mut self) -> mir::BlockId {
        let block = mir::Block::new();
        let id = self.mir.blocks.push(block);
        self.current_block = id;
        id
    }

    pub fn reserve_block(&mut self) -> mir::BlockId {
        self.mir.blocks.reserve()
    }

    pub fn insert_block(&mut self, id: mir::BlockId, block: mir::Block) {
        self.mir.blocks.insert(id, block);
    }

    pub fn finish_block(&mut self, term: mir::Term) {
        self.mir.blocks[self.current_block].term = Some(term);
    }

    pub fn infer(&mut self) -> Result<(), Diagnostic> {
        for local in self.hir.locals.values() {
            self.infer_type(&local.ty, local.id);
        }

        for stmt in self.hir.stmts.values() {
            self.infer_stmt(stmt)?;
        }

        Ok(())
    }

    pub fn infer_stmt(&mut self, stmt: &hir::Stmt) -> Result<(), Diagnostic> {
        match stmt {
            hir::Stmt::Let(stmt) => self.infer_let_stmt(stmt),
            hir::Stmt::Expr(stmt) => self.infer_expr_stmt(stmt),
        }
    }

    pub fn infer_let_stmt(&mut self, stmt: &hir::LetStmt) -> Result<(), Diagnostic> {
        let local = &self.hir.locals[stmt.local];
        let ty = self.infer_type(&local.ty, local.id);

        if let Some(expr) = stmt.init {
            let init_ty = self.infer_expr(&self.hir.exprs[expr])?;
            self.solver.solve(Unify::new(ty, init_ty)).unwrap();
        }

        Ok(())
    }

    pub fn infer_expr_stmt(&mut self, stmt: &hir::ExprStmt) -> Result<(), Diagnostic> {
        self.infer_expr(&self.hir.exprs[stmt.expr])?;
        Ok(())
    }

    pub fn infer_expr(&mut self, expr: &hir::Expr) -> Result<InferType, Diagnostic> {
        match expr {
            hir::Expr::Local(expr) => self.infer_local_expr(expr),
            hir::Expr::Ref(expr) => self.infer_ref_expr(expr),
            hir::Expr::Deref(expr) => self.infer_deref_expr(expr),
            hir::Expr::Assign(expr) => self.infer_assign_expr(expr),
            hir::Expr::Return(expr) => self.infer_return_expr(expr),
        }
    }

    pub fn infer_local_expr(&mut self, expr: &hir::LocalExpr) -> Result<InferType, Diagnostic> {
        let local = &self.hir.locals[expr.local];
        Ok(self.infer_type(&local.ty, local.id))
    }

    pub fn infer_ref_expr(&mut self, expr: &hir::RefExpr) -> Result<InferType, Diagnostic> {
        let pointee = self.infer_expr(&self.hir.exprs[expr.operand])?;
        let ty = InferType::apply(ItemId::Pointer, vec![pointee], expr.span);
        self.types.insert(expr.id, ty.clone());

        Ok(ty)
    }

    pub fn infer_deref_expr(&mut self, expr: &hir::DerefExpr) -> Result<InferType, Diagnostic> {
        let pointee = self.new_type_variable(expr.id);
        let ty = InferType::apply(ItemId::Pointer, vec![pointee.clone()], expr.span);
        let operand = self.infer_expr(&self.hir.exprs[expr.operand])?;

        self.solver.solve(Unify::new(ty, operand)).unwrap();

        Ok(pointee)
    }

    pub fn infer_assign_expr(&mut self, expr: &hir::AssignExpr) -> Result<InferType, Diagnostic> {
        let lhs = self.infer_expr(&self.hir.exprs[expr.lhs])?;
        let rhs = self.infer_expr(&self.hir.exprs[expr.rhs])?;

        self.solver.solve(Unify::new(lhs.clone(), rhs)).unwrap();
        self.types.insert(expr.id, lhs.clone());

        Ok(lhs)
    }

    pub fn infer_return_expr(&mut self, expr: &hir::ReturnExpr) -> Result<InferType, Diagnostic> {
        let ty = if let Some(value) = expr.value {
            self.infer_expr(&self.hir.exprs[value])?
        } else {
            InferType::apply(ItemId::Void, vec![], expr.span)
        };

        let unify = Unify::new(ty.clone(), self.return_type.clone());
        self.solver.solve(unify).unwrap();
        Ok(ty)
    }
}
