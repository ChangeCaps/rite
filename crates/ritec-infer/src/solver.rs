use std::collections::VecDeque;

use ritec_core::{trace, Ident, Span};
use ritec_error::Diagnostic;
use ritec_hir as hir;
use ritec_mir as mir;

use crate::{
    As, Constraint, InferType, InferenceTable, Instance, ItemId, Modification, Normalize,
    Projection, Solution, TypeProjection, TypeVariable, Unify,
};

#[allow(dead_code)]
pub struct Solver<'a> {
    program: &'a hir::Program,
    table: InferenceTable,
    constraints: VecDeque<Constraint>,
    stack: Vec<Constraint>,
    return_type: InferType,
    overflow_depth: usize,
}

impl<'a> Solver<'a> {
    pub fn new(program: &'a hir::Program) -> Self {
        Self {
            program,
            table: InferenceTable::new(),
            constraints: VecDeque::new(),
            stack: Vec::new(),
            return_type: InferType::apply(ItemId::Void, [], Span::DUMMY),
            overflow_depth: 256,
        }
    }

    pub fn finish(mut self) -> Result<InferenceTable, Diagnostic> {
        self.solve_all()?;

        Ok(self.table)
    }

    pub fn table(&self) -> &InferenceTable {
        &self.table
    }

    pub fn table_mut(&mut self) -> &mut InferenceTable {
        &mut self.table
    }

    pub fn return_type(&self) -> &InferType {
        &self.return_type
    }

    pub fn set_return_type(&mut self, ty: hir::Type) {
        self.return_type = self.table.infer_hir(&ty, &Instance::empty());
    }

    pub fn resolve_return_type(&self) -> Result<mir::Type, Diagnostic> {
        self.table.resolve_mir_type(&self.return_type)
    }

    pub fn program(&self) -> &'a hir::Program {
        self.program
    }

    pub fn new_variable(&mut self) -> TypeVariable {
        self.table.new_variable(None)
    }

    fn solve_unify(&mut self, unify: Unify) -> Result<Solution, Diagnostic> {
        let result = self.table.unify(&unify.a, &unify.b)?;
        self.constraints.extend(result.constraints);
        Ok(Solution {
            is_solved: true,
            constraint: Constraint::Unify(unify),
        })
    }

    fn normalize_field(
        &mut self,
        id: hir::HirId,
        base: &InferType,
        field: &Ident,
    ) -> Result<Option<InferType>, Diagnostic> {
        // if base isn't an applied type, try again later
        let InferType::Apply(apply) = base else {
            return Ok(None);
        };

        // if base is a pointer, try dereferencing it
        if let ItemId::Pointer = apply.item {
            if let Some(field) = self.normalize_field(id, &apply[0], field)? {
                self.table.push_modification(id, Modification::Deref);

                return Ok(Some(field));
            }
        }

        // if base isn't a class, it can't have fields
        let ItemId::Class(class_id, _) = apply.item else {
            let err = Diagnostic::error("expected a class")
                .with_msg_span("found this type", apply.span);

            return Err(err);
        };

        let class = &self.program.classes[class_id.cast()];
        trace!("proj: {:?} -> {}", base, class.ident);

        // find the field in the class
        let Some(field) = class.find_field(&field) else {
            let err = Diagnostic::error("field not found")
                .with_msg_span("found this class", class.span)
                .with_msg(format!("class has no field `{}`", field));

            return Err(err);
        };

        self.table.register_field(id, field);

        // create the projection
        let instance = Instance::new(class.generics.params.clone(), apply.arguments.clone());
        Ok(Some(self.table.infer_hir(&class[field].ty, &instance)))
    }

    fn normalize_method(
        &mut self,
        id: hir::HirId,
        base: &InferType,
        method: &Ident,
        generics: &Vec<InferType>,
    ) -> Result<Option<InferType>, Diagnostic> {
        // if base isn't an applied type, try again later
        let InferType::Apply(apply) = base else {
            return Ok(None);
        };

        // if base is a pointer, try dereferencing it
        if let ItemId::Pointer = apply.item {
            if let Some(method) = self.normalize_method(id, &apply[0], method, generics)? {
                self.table.push_modification(id, Modification::Deref);

                return Ok(Some(method));
            }
        }

        // if base isn't a class, it can't have methods
        let ItemId::Class(class_id, _) = apply.item else {
            let err = Diagnostic::error("invalid method access")
                .with_msg_span("method access on non-class type", apply.span);

            return Err(err);
        };

        let class = &self.program.classes[class_id.cast()];
        trace!("proj: {:?} -> {}", base, class.ident);

        // find the method in the class
        let Some(method) = class.find_method(&method) else {
            let err = Diagnostic::error("invalid method access")
                .with_msg_span("method not found in class", apply.span);

            return Err(err);
        };

        self.table.register_method(id, method);

        let method = &class[method];
        let function = &self.program.functions[method.function];

        if matches!(method.self_argument, Some(hir::SelfArgument::Pointer)) {
            self.table.push_modification(id, Modification::Ref);
        }

        let mut fn_generics = apply.arguments.clone();
        if generics.len() == 0 {
            let fn_len = function.generics.params.len();
            let class_len = class.generics.params.len();

            for _ in 0..fn_len - class_len {
                let generic = InferType::Var(self.new_variable());
                self.table.register_generic(id, generic.clone());
                fn_generics.push(generic);
            }
        } else {
            for generic in generics {
                self.table.register_generic(id, generic.clone());
                fn_generics.push(generic.clone());
            }
        }

        if fn_generics.len() != function.generics.params.len() {
            let err = Diagnostic::error("invalid method access").with_msg_span(
                format!(
                    "wrong number of generic arguments, expected {}, got {}",
                    function.generics.params.len() - class.generics.params.len(),
                    fn_generics.len() - class.generics.params.len()
                ),
                apply.span,
            );

            return Err(err);
        }

        let instance = Instance::new(function.generics.params.clone(), fn_generics);

        let mut function = function.ty();
        function.arguments.remove(0);

        let ty = hir::Type::Function(function);
        Ok(Some(self.table.infer_hir(&ty, &instance)))
    }

    fn normalize_projection(
        &mut self,
        proj: &TypeProjection,
    ) -> Result<Option<InferType>, Diagnostic> {
        if let Some(ty) = self.table.normalize_shallow(&InferType::Proj(proj.clone())) {
            return Ok(Some(ty));
        }

        if let Some(ty) = self.normalize(&proj.base)? {
            let res = self.normalize_projection(&TypeProjection {
                base: Box::new(ty),
                proj: proj.proj.clone(),
            })?;

            if let Some(ty) = res.clone() {
                (self.table).substitute(InferType::Proj(proj.clone()), ty);
            }

            return Ok(res);
        }

        let res = match proj.proj {
            Projection::Field(id, ref field) => self.normalize_field(id, &proj.base, field)?,
            Projection::Method(id, ref method, ref generics) => {
                self.normalize_method(id, &proj.base, method, generics)?
            }
        };

        if let Some(res) = res.clone() {
            self.table.substitute(InferType::Proj(proj.clone()), res);
        }

        Ok(res)
    }

    fn solve_normalize(&mut self, norm: Normalize) -> Result<Solution, Diagnostic> {
        trace!("normalize: {:?} = {:?}", norm.proj, norm.expected);

        let Some(ty) = self.normalize_projection(&norm.proj)? else {
            return Ok(Solution {
                is_solved: false,
                constraint: Constraint::Normalize(norm),
            });
        };

        self.unify(ty, norm.expected)
    }

    fn solve_as(&mut self, ty: &InferType, expected: &InferType) -> Result<Solution, Diagnostic> {
        trace!("as: {:?} as {:?}", ty, expected);

        // always apply substitutions
        if let Some(ty) = self.normalize(&ty)? {
            return self.solve_as(&ty, expected);
        } else if let Some(expected) = self.normalize(&expected)? {
            return self.solve_as(ty, &expected);
        }

        // if ty == expected, we're done
        if ty == expected {
            return Ok(Solution {
                is_solved: true,
                constraint: Constraint::As(As::new(ty.clone(), expected.clone())),
            });
        }

        let (InferType::Apply(ty), InferType::Apply(expected)) = (ty, expected) else {
            return Ok(Solution {
                is_solved: false,
                constraint: Constraint::As(As::new(ty.clone(), expected.clone())),
            });
        };

        match (&ty.item, &expected.item) {
            /* pointer to pointer */
            (ItemId::Pointer, ItemId::Pointer) => {}

            /* pointer to int */
            (ItemId::Int(_), ItemId::Pointer) => {}
            (ItemId::Pointer, ItemId::Int(_)) => {}

            /* int to int */
            (ItemId::Int(_), ItemId::Int(_)) => {}

            /* float to int */
            (ItemId::Float(_), ItemId::Int(_)) => {}
            (ItemId::Int(_), ItemId::Float(_)) => {}

            /* float to float */
            (ItemId::Float(_), ItemId::Float(_)) => {}
            _ => {
                let err = Diagnostic::error("invalid type cast")
                    .with_msg_span("type cast on non-class type", Span::DUMMY);

                return Err(err);
            }
        }

        Ok(Solution {
            is_solved: true,
            constraint: Constraint::As(As::new(ty.clone(), expected.clone())),
        })
    }

    fn normalize(&mut self, ty: &InferType) -> Result<Option<InferType>, Diagnostic> {
        if let Some(ty) = self.table.normalize_shallow(&ty) {
            return Ok(Some(ty));
        }

        match ty {
            InferType::Proj(proj) => self.normalize_projection(proj),
            _ => Ok(None),
        }
    }

    pub fn solve(&mut self, constraint: impl Into<Constraint>) -> Result<Solution, Diagnostic> {
        let constraint = constraint.into();

        if self.stack.contains(&constraint) || self.stack.len() > self.overflow_depth {
            return Ok(Solution {
                is_solved: false,
                constraint,
            });
        }

        self.stack.push(constraint.clone());

        let result = match constraint {
            Constraint::Unify(unify) => self.solve_unify(unify),
            Constraint::Normalize(norm) => self.solve_normalize(norm),
            Constraint::As(as_) => self.solve_as(&as_.ty, &as_.expected),
        };

        self.stack.pop().unwrap();

        let solution = result?;

        if !solution.is_solved {
            self.constraints.push_back(solution.constraint.clone());
        }

        Ok(solution)
    }

    pub fn solve_all(&mut self) -> Result<(), Diagnostic> {
        while let Some(constraint) = self.constraints.pop_front() {
            self.solve(constraint)?;
        }

        Ok(())
    }

    pub fn unify(
        &mut self,
        a: impl Into<InferType>,
        b: impl Into<InferType>,
    ) -> Result<Solution, Diagnostic> {
        self.solve(Constraint::Unify(Unify::new(a, b)))
    }

    pub fn register_type(&mut self, id: hir::HirId, hir: &hir::Type) -> InferType {
        self.table.register_hir(id, hir)
    }
}
