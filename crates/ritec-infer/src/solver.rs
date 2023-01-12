use ritec_ir::Program;

use crate::{Constraint, InferError, InferType, InferenceTable, Solution, TypeVariable, Unify};

pub struct Solver<'a> {
    program: &'a Program,
    table: InferenceTable,
    constraints: Vec<Constraint>,
    stack: Vec<Constraint>,
    overflow_depth: usize,
}

impl<'a> Solver<'a> {
    pub fn new(program: &'a Program) -> Self {
        Self {
            program,
            table: InferenceTable::new(),
            constraints: Vec::new(),
            stack: Vec::new(),
            overflow_depth: 256,
        }
    }

    pub fn next_variable(&mut self) -> TypeVariable {
        self.table.next_variable()
    }

    pub fn finish(self) -> InferenceTable {
        self.table
    }

    fn unify(&mut self, a: &InferType, b: &InferType) -> Result<Solution, InferError> {
        let result = self.table.unify(a, b)?;
        self.constraints.extend(result.constraints);
        Ok(Solution {
            is_solved: true,
            constraint: Constraint::Unify(Unify::new(a.clone(), b.clone())),
        })
    }

    fn solve_one(
        &mut self,
        constraint: &Constraint,
        progress: &mut bool,
    ) -> Result<(), InferError> {
        let solution = self.solve(constraint.clone())?;

        todo!()
    }

    pub fn solve(&mut self, constraint: impl Into<Constraint>) -> Result<Solution, InferError> {
        let constraint = constraint.into();

        if self.stack.contains(&constraint) || self.stack.len() > self.overflow_depth {
            return Ok(Solution {
                is_solved: false,
                constraint,
            });
        }

        self.stack.push(constraint.clone());

        let result = match constraint {
            Constraint::Unify(unify) => self.unify(&unify.a, &unify.b),
            Constraint::Normalize(_) => todo!(),
        };

        self.stack.pop().unwrap();

        result
    }
}
