use std::fmt::Display;

use ritec_core::Id;

use crate::{Statement, Terminator};

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
        self.terminator = Some(terminator);
    }

    pub fn push(&mut self, stmt: Statement) {
        self.statements.push(stmt);
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
