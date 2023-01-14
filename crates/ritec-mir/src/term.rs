use std::fmt::{self, Display};

use crate::Operand;

#[derive(Clone, Debug, PartialEq)]
pub enum Term {
    Return(Operand),
}

impl Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Return(operand) => write!(f, "return {}", operand),
        }
    }
}
