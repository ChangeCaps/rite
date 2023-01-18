use ritec_core::{trace, Span};
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
    constraints: Vec<Constraint>,
    stack: Vec<Constraint>,
    return_type: InferType,
    overflow_depth: usize,
}

impl<'a> Solver<'a> {
    pub fn new(program: &'a hir::Program) -> Self {
        Self {
            program,
            table: InferenceTable::new(),
            constraints: Vec::new(),
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

    fn normalize_projection(&mut self, proj: &TypeProjection) -> Option<InferType> {
        if let Some(ty) = self.table.normalize_shallow(&InferType::Proj(proj.clone())) {
            return Some(ty);
        }

        if let Some(ty) = self.table.normalize_shallow(&proj.base) {
            let proj = TypeProjection {
                base: Box::new(ty),
                proj: proj.proj.clone(),
            };

            return self.normalize_projection(&proj);
        }

        match proj.proj {
            Projection::Field(ref field) => {
                let InferType::Apply(apply) = proj.base.as_ref() else {
                    return None;
                };

                let ItemId::Class(class_id, _) = apply.item else {
                    return None;
                };

                let class = &self.program.classes[class_id.cast()];
                trace!("proj: {:?} -> {}", proj.base, class.ident);

                let field = class.find_field(&field)?;
                trace!("proj: {:?} -> {}", proj, class[field].ty);

                let mut generics = Vec::new();
                for _ in 0..class.generics.params.len() {
                    generics.push(InferType::Var(self.table.new_variable(None)));
                }

                let instance = Instance::new(class.generics.params.clone(), generics);
                Some(self.table.infer_hir(&class[field].ty, &instance))
            }
        }
    }

    fn solve_normalize(&mut self, norm: Normalize) -> Result<Solution, Error> {
        trace!("normalize: {:?} = {:?}", norm.proj, norm.expected);

        let Some(ty) = self.normalize_projection(&norm.proj) else {
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
            self.constraints.push(solution.constraint.clone());
        }

        Ok(solution)
    }

    pub fn solve_all(&mut self) -> Result<(), Error> {
        while let Some(constraint) = self.constraints.pop() {
            let solution = self.solve(constraint)?;

            if !solution.is_solved {
                self.constraints.insert(0, solution.constraint);
            }
        }

        Ok(())
    }

    pub fn unify(
        &mut self,
        a: impl Into<InferType>,
        b: impl Into<InferType>,
    ) -> Result<Solution, Error> {
        self.solve(Constraint::Unify(Unify::new(a.into(), b.into())))
    }

    pub fn register_type(&mut self, id: hir::HirId, hir: &hir::Type) -> InferType {
        self.table.register_hir(id, hir)
    }
}
