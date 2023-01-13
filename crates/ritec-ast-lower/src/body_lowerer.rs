use ritec_ast as ast;
use ritec_core::Ident;
use ritec_error::Diagnostic;
use ritec_hir as hir;

use crate::TypeLowerer;

pub struct BodyLowerer<'a> {
    pub body: &'a mut hir::Body,
    pub type_lowerer: TypeLowerer<'a>,
    pub scope: Vec<hir::LocalId>,
}

impl<'a> BodyLowerer<'a> {
    pub fn new(body: &'a mut hir::Body, type_lowerer: TypeLowerer<'a>) -> Self {
        let scope = body.locals.keys().collect();

        Self {
            body,
            type_lowerer,
            scope,
        }
    }

    pub fn lower_type(&self, ty: &ast::Type) -> Result<hir::Type, Diagnostic> {
        self.type_lowerer.lower_type(ty)
    }

    pub fn lower_block(&mut self, block: &ast::Block) -> Result<(), Diagnostic> {
        for stmt in block.stmts.iter() {
            self.lower_stmt(stmt)?;
        }

        Ok(())
    }

    pub fn lower_stmt(&mut self, stmt: &ast::Stmt) -> Result<hir::StmtId, Diagnostic> {
        let stmt = match stmt {
            ast::Stmt::Let(stmt) => self.lower_let_stmt(stmt)?,
        };

        Ok(self.body.stmts.push(stmt))
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
            id: self.body.next_universe_id(),
        };

        let init = if let Some(ref init) = stmt.init {
            Some(self.lower_expr(init)?)
        } else {
            None
        };

        let let_stmt = hir::LetStmt {
            local: self.body.locals.push(local),
            init,
            id: self.body.next_universe_id(),
            span: stmt.span,
        };

        Ok(hir::Stmt::Let(let_stmt))
    }

    pub fn lower_expr(&mut self, expr: &ast::Expr) -> Result<hir::ExprId, Diagnostic> {
        let expr = match expr {
            ast::Expr::Path(expr) => self.lower_path_expr(expr)?,
            ast::Expr::Unary(expr) => self.lower_unary_expr(expr)?,
        };

        Ok(self.body.exprs.push(expr))
    }

    pub fn lower_unary_expr(&mut self, expr: &ast::UnaryExpr) -> Result<hir::Expr, Diagnostic> {
        match expr.operator {
            ast::UnaryOp::Ref => self.lower_ref_expr(expr),
            ast::UnaryOp::Deref => self.lower_deref_expr(expr),
        }
    }

    pub fn lower_ref_expr(&mut self, expr: &ast::UnaryExpr) -> Result<hir::Expr, Diagnostic> {
        let ref_expr = hir::RefExpr {
            operand: self.lower_expr(&expr.operand)?,
            id: self.body.next_universe_id(),
            span: expr.span,
        };

        Ok(hir::Expr::Ref(ref_expr))
    }

    pub fn lower_deref_expr(&mut self, expr: &ast::UnaryExpr) -> Result<hir::Expr, Diagnostic> {
        let deref_expr = hir::DerefExpr {
            operand: self.lower_expr(&expr.operand)?,
            id: self.body.next_universe_id(),
            span: expr.span,
        };

        Ok(hir::Expr::Deref(deref_expr))
    }

    pub fn lower_path_expr(&mut self, expr: &ast::PathExpr) -> Result<hir::Expr, Diagnostic> {
        if let Some(ident) = expr.path.get_ident() {
            if let Some(local) = self.find_local(ident) {
                let local_expr = hir::LocalExpr {
                    id: self.body.next_universe_id(),
                    local,
                    span: expr.span(),
                };

                return Ok(hir::Expr::Local(local_expr));
            }
        }

        let err = Diagnostic::error("expected a local variable")
            .with_message_span("variable not found", expr.span());

        Err(err)
    }

    pub fn find_local(&self, ident: &Ident) -> Option<hir::LocalId> {
        self.body
            .locals
            .iter()
            .rev()
            .find(|(_, local)| local.ident == *ident)
            .map(|(id, _)| id)
    }
}
