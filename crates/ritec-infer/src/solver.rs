use std::collections::VecDeque;

use ritec_core::{trace, Ident, Span};
use ritec_hir as hir;
use ritec_mir as mir;

use crate::{
    Constraint, Error, InferType, InferenceTable, Instance, ItemId, Normalize, Projection,
    Solution, TypeProjection, TypeVariable, Unify,
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

    pub fn finish(mut self) -> Result<InferenceTable, Error> {
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

    pub fn resolve_return_type(&self) -> Result<mir::Type, Error> {
        self.table.resolve_mir_type(&self.return_type)
    }

    pub fn program(&self) -> &'a hir::Program {
        self.program
    }

    pub fn new_variable(&mut self) -> TypeVariable {
        self.table.new_variable(None)
    }

    fn solve_unify(&mut self, unify: Unify) -> Result<Solution, Error> {
        let result = self.table.unify(&unify.a, &unify.b)?;
        self.constraints.extend(result.constraints);
        Ok(Solution {
            is_solved: true,
            constraint: Constraint::Unify(unify),
        })
    }

    fn normalize_field(
        &mut self,
        base: &InferType,
        field: &Ident,
    ) -> Result<Option<InferType>, Error> {
        let InferType::Apply(apply) = base else {
            return Ok(None);
        };

        let ItemId::Class(class_id, _) = apply.item else {
            return Err(Error::InvalidFieldAccess(apply.clone(), field.clone()));
        };

        let class = &self.program.classes[class_id.cast()];
        trace!("proj: {:?} -> {}", base, class.ident);

        let Some(field) = class.find_field(&field) else {
            return Err(Error::InvalidFieldAccess(apply.clone(), field.clone()));
        };

        let instance = Instance::new(class.generics.params.clone(), apply.arguments.clone());
        Ok(Some(self.table.infer_hir(&class[field].ty, &instance)))
    }

    fn normalize_projection(&mut self, proj: &TypeProjection) -> Result<Option<InferType>, Error> {
        if let Some(ty) = self.table.normalize_shallow(&InferType::Proj(proj.clone())) {
            return Ok(Some(ty));
        }

        if let Some(ty) = self.table.normalize_shallow(&proj.base) {
            let proj = TypeProjection {
                base: Box::new(ty),
                proj: proj.proj.clone(),
            };

            return self.normalize_projection(&proj);
        }

        match proj.proj {
            Projection::Field(ref field) => self.normalize_field(&proj.base, field),
        }
    }

    fn solve_normalize(&mut self, norm: Normalize) -> Result<Solution, Error> {
        trace!("normalize: {:?} = {:?}", norm.proj, norm.expected);

        let Some(ty) = self.normalize_projection(&norm.proj)? else {
            return Ok(Solution {
                is_solved: false,
                constraint: Constraint::Normalize(norm),
            });
        };

        (self.table).substitute(InferType::Proj(norm.proj), ty.clone());
        self.unify(ty, norm.expected)
    }

    pub fn solve(&mut self, constraint: impl Into<Constraint>) -> Result<Solution, Error> {
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
        };

        self.stack.pop().unwrap();

        let solution = result?;

        if !solution.is_solved {
            self.constraints.push_front(solution.constraint.clone());
        }

        Ok(solution)
    }

    pub fn solve_all(&mut self) -> Result<(), Error> {
        while let Some(constraint) = self.constraints.pop_front() {
            let solution = self.solve(constraint)?;

            if !solution.is_solved {
                self.constraints.push_back(solution.constraint);
            }
        }

        Ok(())
    }

    pub fn unify(
        &mut self,
        a: impl Into<InferType>,
        b: impl Into<InferType>,
    ) -> Result<Solution, Error> {
        self.solve(Constraint::Unify(Unify::new(a, b)))
    }

    pub fn normalize(
        &mut self,
        proj: impl Into<TypeProjection>,
        expected: impl Into<InferType>,
    ) -> Result<Solution, Error> {
        self.solve(Constraint::Normalize(Normalize::new(proj, expected)))
    }

    pub fn register_type(&mut self, id: hir::HirId, hir: &hir::Type) -> InferType {
        self.table.register_hir(id, hir)
    }
}
