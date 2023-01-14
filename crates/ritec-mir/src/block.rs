use std::fmt::Display;

use ritec_core::Id;

use crate::{Stmt, Term};

pub type BlockId = Id<Block>;

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub term: Option<Term>,
}

impl Block {
    pub const fn new() -> Self {
        Self {
            stmts: Vec::new(),
            term: None,
        }
    }

    pub fn push(&mut self, stmt: Stmt) {
        self.stmts.push(stmt);
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;

        for stmt in &self.stmts {
            writeln!(f, "\t\t{}", stmt)?;
        }

        if let Some(term) = &self.term {
            writeln!(f, "\t\t{}", term)?;
        }

        write!(f, "\t}}")
    }
}
