use std::fmt::Display;

use ritec_core::Id;

use crate::{Assign, Operand, Place, Statement, SwitchTargets, Terminator, Value};

pub type BlockId = Id<Block>;

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub terminator: Option<Terminator>,
}

impl Block {
    pub const fn new() -> Self {
        Self {
            statements: Vec::new(),
            terminator: None,
        }
    }

    pub fn is_terminated(&self) -> bool {
        self.terminator.is_some()
    }

    pub fn terminate(&mut self, terminator: Terminator) {
        if self.is_terminated() {
            return;
        }

        self.terminator = Some(terminator);
    }

    pub fn is_empty(&self) -> bool {
        self.statements.is_empty() && self.terminator.is_none()
    }

    pub fn push(&mut self, stmt: Statement) {
        self.statements.push(stmt);
    }

    pub fn push_assign(&mut self, place: impl Into<Place>, value: impl Into<Value>) {
        let assign = Assign {
            place: place.into(),
            value: value.into(),
        };

        self.push(Statement::Assign(assign));
    }

    pub fn push_drop(&mut self, value: impl Into<Value>) {
        self.push(Statement::Drop(value.into()));
    }

    pub fn terminate_return(&mut self, value: impl Into<Operand>) {
        self.terminate(Terminator::Return(value.into()));
    }

    pub fn terminate_goto(&mut self, target: BlockId) {
        self.terminate(Terminator::Goto(target));
    }

    pub fn terminate_switch(&mut self, value: impl Into<Operand>, targets: SwitchTargets) {
        self.terminate(Terminator::Switch(value.into(), targets));
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;

        for stmt in &self.statements {
            writeln!(f, "\t\t{}", stmt)?;
        }

        if let Some(term) = &self.terminator {
            writeln!(f, "\t\t{}", term)?;
        }

        write!(f, "\t}}")
    }
}
