use std::fmt::{self, Display};

use crate::{BlockId, Operand};

#[derive(Clone, Debug, PartialEq)]
pub enum Terminator {
    Return(Operand),
}

impl Terminator {
    pub fn successors(&self) -> Vec<BlockId> {
        match self {
            _ => Vec::new(),
        }
    }
}

impl Display for Terminator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Return(operand) => write!(f, "return {}", operand),
        }
    }
}
