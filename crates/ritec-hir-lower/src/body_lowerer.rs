use std::collections::HashMap;

use ritec_error::Diagnostic;
use ritec_hir as hir;
use ritec_infer::{InferType, ItemId, Solver};
use ritec_mir as mir;

pub struct BodyLowerer<'a> {
    pub hir: &'a hir::Body,
    pub mir: &'a mut mir::Body,
    pub return_type: InferType,
    pub current_block: mir::BlockId,
    pub solver: Solver<'a>,
    pub types: HashMap<hir::UniverseId, InferType>,
}

impl<'a> BodyLowerer<'a> {
    pub fn new(
        hir: &'a hir::Body,
        mir: &'a mut mir::Body,
        return_type: &hir::Type,
        mut solver: Solver<'a>,
    ) -> Self {
        let block = mir.blocks.push(mir::Block::new());

        Self {
            hir,
            mir,
            return_type: solver.infer_type(return_type),
            current_block: block,
            solver,
            types: HashMap::new(),
        }
    }

    fn lower_item_id(&self, item: &ItemId, mut arguments: Vec<mir::Type>) -> mir::Type {
        match item {
            ItemId::Void => mir::Type::VOID,
            ItemId::Bool => mir::Type::BOOL,
            ItemId::Int(ty) => mir::Type::Int(ty.clone()),
            ItemId::Float(ty) => mir::Type::Float(ty.clone()),
            ItemId::Pointer => {
                let pointee = arguments.pop().unwrap();
                mir::Type::Pointer(mir::PointerType {
                    pointee: Box::new(pointee),
                })
            }
            ItemId::Array(size) => {
                let element = arguments.pop().unwrap();
                mir::Type::Array(mir::ArrayType {
                    element: Box::new(element),
                    size: *size,
                })
            }
            ItemId::Slice => {
                let element = arguments.pop().unwrap();
                mir::Type::Slice(mir::SliceType {
                    element: Box::new(element),
                })
            }
            ItemId::Function => {
                let return_type = arguments.pop().unwrap();

                mir::Type::Function(mir::FunctionType {
                    arguments,
                    return_type: Box::new(return_type),
                })
            }
            ItemId::Tuple => mir::Type::Tuple(mir::TupleType { fields: arguments }),
            ItemId::Generic(ref generic) => mir::Type::Generic(mir::GenericType {
                generic: generic.clone(),
            }),
        }
    }

    pub fn lower_type(&self, ty: &InferType) -> Result<mir::Type, Diagnostic> {
        match ty {
            InferType::Var(var) => {
                if let Some(ty) = self.solver.get_substitution(var) {
                    self.lower_type(&ty)
                } else {
                    println!("{:#?}", self.solver.table());

                    let err = Diagnostic::error("ambiguous type");
                    return Err(err);
                }
            }
            InferType::Apply(apply) => {
                let mut arguments = Vec::with_capacity(apply.arguments.len());
                for argument in &apply.arguments {
                    arguments.push(self.lower_type(argument)?);
                }

                Ok(self.lower_item_id(&apply.item, arguments))
            }
            InferType::Proj(_) => todo!(),
        }
    }

    pub fn inferred_type(&self, ty: &hir::UniverseId) -> Result<mir::Type, Diagnostic> {
        let Some(ty) = self.types.get(ty)  else {
            unreachable!("universe id not registered, compiler error {:?}", ty)
        };

        self.lower_type(ty)
    }

    pub fn block_mut(&mut self, id: mir::BlockId) -> &mut mir::Block {
        &mut self.mir.blocks[id]
    }

    pub fn push(&mut self, stmt: mir::Stmt) {
        self.block_mut(self.current_block).stmts.push(stmt);
    }

    pub fn push_temp(&mut self, value: mir::Value, ty: impl Into<mir::Type>) -> mir::LocalId {
        let local = mir::Local {
            ident: None,
            ty: ty.into(),
        };

        let local = self.mir.locals.push(local);
        let assign = mir::Assign {
            place: mir::Place::from(local),
            value,
        };

        self.push(mir::Stmt::Assign(assign));

        local
    }

    pub fn lower(&mut self) -> Result<(), Diagnostic> {
        for (hir_id, local) in self.hir.locals.iter() {
            let ty = self.inferred_type(&local.id)?;
            let local = mir::Local {
                ident: Some(local.ident.clone()),
                ty,
            };
            self.mir.locals.insert(hir_id.cast(), local);
        }

        for stmt in self.hir.stmts.values() {
            self.lower_stmt(stmt)?;
        }

        Ok(())
    }

    pub fn lower_stmt(&mut self, stmt: &hir::Stmt) -> Result<(), Diagnostic> {
        match stmt {
            hir::Stmt::Let(stmt) => self.lower_let_stmt(stmt),
            hir::Stmt::Expr(stmt) => self.lower_expr_stmt(stmt),
        }
    }

    pub fn lower_let_stmt(&mut self, stmt: &hir::LetStmt) -> Result<(), Diagnostic> {
        if let Some(init) = stmt.init {
            let value = self.lower_expr(&self.hir.exprs[init])?;
            let assign = mir::Assign {
                place: mir::Place::from(stmt.local.cast()),
                value,
            };
            self.push(mir::Stmt::Assign(assign));
        }

        Ok(())
    }

    pub fn lower_expr_stmt(&mut self, stmt: &hir::ExprStmt) -> Result<(), Diagnostic> {
        self.lower_expr(&self.hir.exprs[stmt.expr])?;

        Ok(())
    }

    pub fn lower_expr(&mut self, expr: &hir::Expr) -> Result<mir::Value, Diagnostic> {
        match expr {
            hir::Expr::Local(expr) => self.lower_local_expr(expr),
            hir::Expr::Ref(expr) => self.lower_ref_expr(expr),
            hir::Expr::Deref(expr) => self.lower_deref_expr(expr),
            hir::Expr::Assign(expr) => self.lower_assign_expr(expr),
            hir::Expr::Return(expr) => self.lower_return_expr(expr),
        }
    }

    pub fn lower_local_expr(&mut self, expr: &hir::LocalExpr) -> Result<mir::Value, Diagnostic> {
        let place = mir::Place::from(expr.local.cast());
        Ok(mir::Value::Operand(mir::Operand::Copy(place)))
    }

    pub fn lower_ref_expr(&mut self, expr: &hir::RefExpr) -> Result<mir::Value, Diagnostic> {
        let value = self.lower_expr(&self.hir.exprs[expr.operand])?;

        if let Some(place) = value.as_place() {
            return Ok(mir::Value::Address(place.clone()));
        }

        let ty = self.inferred_type(&expr.id)?;
        let local = self.push_temp(value, ty);

        Ok(mir::Value::Address(mir::Place::from(local)))
    }

    pub fn lower_deref_expr(&mut self, expr: &hir::DerefExpr) -> Result<mir::Value, Diagnostic> {
        let value = self.lower_expr(&self.hir.exprs[expr.operand])?;

        let Some(mut place) = value.to_place() else {
            let err = Diagnostic::error("invalid dereference")
                .with_message_span("cannot dereference expression", self.hir.exprs[expr.operand].span());

            return Err(err);
        };

        place.proj.push(mir::Projection::Deref);

        Ok(mir::Value::Operand(mir::Operand::Copy(place.clone())))
    }

    pub fn lower_assign_expr(&mut self, expr: &hir::AssignExpr) -> Result<mir::Value, Diagnostic> {
        let place = self.lower_expr(&self.hir.exprs[expr.lhs])?;

        let Some(place) = place.to_place() else {
            let err = Diagnostic::error("invalid assignment")
                .with_message_span("cannot assign to this expression", self.hir.exprs[expr.lhs].span());

            return Err(err);
        };

        let remain = self.push_temp(
            mir::Value::move_operand(place.clone()),
            self.inferred_type(&expr.id)?,
        );

        let value = self.lower_expr(&self.hir.exprs[expr.rhs])?;
        let assign = mir::Assign { place, value };
        self.push(mir::Stmt::Assign(assign));

        Ok(mir::Value::move_operand(remain))
    }

    pub fn lower_return_expr(&mut self, expr: &hir::ReturnExpr) -> Result<mir::Value, Diagnostic> {
        let value = if let Some(value) = expr.value {
            self.lower_expr(&self.hir.exprs[value])?
        } else {
            mir::Value::VOID
        };

        if let Some(operand) = value.as_operand() {
            self.finish_block(mir::Term::Return(operand.clone()));

            return Ok(mir::Value::VOID);
        };

        let local = self.push_temp(value, self.inferred_type(&expr.id)?);
        self.finish_block(mir::Term::Return(mir::Operand::Move(local.into())));
        Ok(mir::Value::VOID)
    }
}
