use ritec_core::Span;
use ritec_hir as hir;
use ritec_mir as mir;

use crate::{
    Constraint, Error, InferType, InferenceTable, Instance, ItemId, Solution, TypeVariable, Unify,
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

    pub fn program(&self) -> &hir::Program {
        self.program
    }

    pub fn new_variable(&mut self) -> TypeVariable {
        self.table.new_variable(None)
    }

    pub fn get_substitution(&self, var: &TypeVariable) -> Option<InferType> {
        self.table.get_substitution(var)
    }

    fn solve_unify(&mut self, a: &InferType, b: &InferType) -> Result<Solution, Error> {
        let result = self.table.unify(a, b)?;
        self.constraints.extend(result.constraints);
        Ok(Solution {
            is_solved: true,
            constraint: Constraint::Unify(Unify::new(a.clone(), b.clone())),
        })
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
            Constraint::Unify(unify) => self.solve_unify(&unify.a, &unify.b),
            Constraint::Normalize(_) => todo!(),
        };

        self.stack.pop().unwrap();

        result
    }

    pub fn solve_all(&mut self) -> Result<(), Error> {
        while let Some(constraint) = self.constraints.pop() {
            let solution = self.solve(constraint)?;

            if !solution.is_solved {
                self.constraints.push(solution.constraint);
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
