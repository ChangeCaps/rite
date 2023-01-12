use ritec_ast as ast;
use ritec_infer::{InferType, InferenceTable, TypeVariable};
use ritec_ir::{Body, Expr, ExprId, ExprKind, LocalExpr, LocalId, Program, Type};

use crate::LowerError;

pub struct BodyLowerer<'a> {
    pub program: &'a Program,
    pub infer: InferenceTable,
    pub next_variable: usize,
    pub body: &'a mut Body,
    pub scope: Vec<LocalId>,
}

impl<'a> BodyLowerer<'a> {
    pub fn new(program: &'a Program, infer: InferenceTable, body: &'a mut Body) -> Self {
        Self {
            program,
            infer,
            next_variable: 0,
            body,
            scope: Vec::new(),
        }
    }

    pub fn lower_block(&mut self, block: &ast::Block) -> Result<(), LowerError> {
        todo!()
    }

    pub fn lower_stmt(&mut self, stmt: &ast::Stmt) -> Result<(), LowerError> {
        todo!()
    }

    pub fn lower_let_stmt(&mut self, stmt: &ast::LetStmt) -> Result<(), LowerError> {
        let ty = if let Some(ref ty) = stmt.ty {
            self.infer_type(ty)?
        } else {
            InferType::Var(self.solver.next_variable())
        };

        todo!()
    }

    pub fn lower_expr(&mut self, expr: &ast::Expr) -> Result<ExprId, LowerError> {
        let expr = match expr {
            ast::Expr::Path(expr) => self.lower_path_expr(expr)?,
        };

        Ok(self.body.exprs.push(expr))
    }

    pub fn lower_path_expr(&mut self, expr: &ast::PathExpr) -> Result<Expr, LowerError> {
        let Some(ident) = expr.path.get_ident() else {
            return Err(LowerError::InvalidPath(expr.path.clone()));
        };

        for &local_id in self.scope.iter() {
            let local = &self.body.locals[local_id];
            if local.ident == *ident {
                let local_expr = LocalExpr { local: local_id };

                return Ok(Expr {
                    kind: ExprKind::Local(local_expr),
                    ty: local.ty.clone(),
                    span: expr.span(),
                });
            }
        }

        todo!()
    }

    pub fn next_variable(&mut self) -> TypeVariable {
        let var = TypeVariable::new(self.next_variable);
        self.next_variable += 1;
        var
    }

    pub fn infer_type(&mut self, ty: &ast::Type) -> Result<Type, LowerError> {
        match ty.kind {
            ast::TypeKind::Inferred => {
                let var = self.next_variable();
                Ok(InferType::Var(var))
            }
            ast::TypeKind::Void => todo!(),
            ast::TypeKind::Bool => todo!(),
            ast::TypeKind::Int(_) => todo!(),
            ast::TypeKind::Float(_) => todo!(),
            ast::TypeKind::Pointer(_) => todo!(),
            ast::TypeKind::Array(_) => todo!(),
            ast::TypeKind::Slice(_) => todo!(),
            ast::TypeKind::Function(_) => todo!(),
            ast::TypeKind::Tuple(_) => todo!(),
            ast::TypeKind::Path(_) => todo!(),
        }
    }
}
