use ritec_ast as ast;
use ritec_infer::{InferType, ItemId, Solver, TypeApplication, Unify};
use ritec_ir::Program;
use ritec_span::Ident;

use crate::LowerError;

pub struct InferLocal {
    pub ident: Ident,
    pub ty: InferType,
}

pub struct BodyInferer<'a> {
    pub program: &'a Program,
    pub solver: Solver<'a>,
    pub locals: Vec<InferLocal>,
}

impl<'a> BodyInferer<'a> {
    pub fn new(program: &'a Program) -> Self {
        Self {
            program,
            solver: Solver::new(program),
            locals: Vec::new(),
        }
    }

    pub fn infer_block(&mut self, block: &ast::Block) -> Result<(), LowerError> {
        let index = self.locals.len();

        for stmt in block.stmts.iter() {
            self.infer_stmt(stmt)?;
        }

        self.locals.truncate(index);

        Ok(())
    }

    pub fn infer_stmt(&mut self, stmt: &ast::Stmt) -> Result<(), LowerError> {
        match stmt {
            ast::Stmt::Let(stmt) => self.infer_let_stmt(stmt),
        }
    }

    pub fn infer_let_stmt(&mut self, stmt: &ast::LetStmt) -> Result<(), LowerError> {
        let ty = if let Some(ref ty) = stmt.ty {
            self.infer_type(ty)?
        } else {
            InferType::Var(self.solver.next_variable())
        };

        self.locals.push(InferLocal {
            ident: stmt.ident.clone(),
            ty: ty.clone(),
        });

        if let Some(ref expr) = stmt.init {
            let init_ty = self.infer_expr(expr)?;
            self.solver.solve(Unify::new(ty, init_ty))?;
        }

        Ok(())
    }

    pub fn infer_expr(&mut self, expr: &ast::Expr) -> Result<InferType, LowerError> {
        match expr {
            ast::Expr::Path(path_expr) => self.infer_path_expr(path_expr),
        }
    }

    pub fn infer_path_expr(&mut self, expr: &ast::PathExpr) -> Result<InferType, LowerError> {
        let Some(ident) = expr.path.get_ident() else {
            return Err(LowerError::InvalidPath(expr.path.clone()));
        };

        if let Some(local) = self.locals.iter().find(|local| local.ident == *ident) {
            return Ok(local.ty.clone());
        }

        todo!()
    }

    pub fn infer_type(&mut self, ty: &ast::Type) -> Result<InferType, LowerError> {
        let apply = match ty.kind {
            ast::TypeKind::Inferred => return Ok(InferType::Var(self.solver.next_variable())),
            ast::TypeKind::Void => TypeApplication::new(ItemId::Void),
            ast::TypeKind::Bool => TypeApplication::new(ItemId::Bool),
            ast::TypeKind::Int(_) => todo!(),
            ast::TypeKind::Float(_) => todo!(),
            ast::TypeKind::Pointer(_) => todo!(),
            ast::TypeKind::Array(_) => todo!(),
            ast::TypeKind::Slice(_) => todo!(),
            ast::TypeKind::Function(_) => todo!(),
            ast::TypeKind::Tuple(_) => todo!(),
            ast::TypeKind::Path(_) => todo!(),
        };

        Ok(InferType::Apply(apply))
    }
}
