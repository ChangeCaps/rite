use std::fmt::{self, Display};

use crate::{BlockId, Operand};

#[derive(Clone, Debug, PartialEq)]
pub enum Term {
    Return(Operand),
}

impl Term {
    pub fn successors(&self) -> Vec<BlockId> {
        match self {
            _ => Vec::new(),
        }
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Return(operand) => write!(f, "return {}", operand),
        }
    }
}
