use std::fmt::{self, Display};

use crate::{Constant, Place};

#[derive(Clone, Debug, PartialEq)]
pub enum Operand {
    Copy(Place),
    Move(Place),
    Constant(Constant),
}

impl Operand {
    pub const VOID: Self = Self::Constant(Constant::Void);

    pub fn as_place(&self) -> Option<&Place> {
        match self {
            Operand::Copy(place) => Some(place),
            Operand::Move(place) => Some(place),
            Operand::Constant(_) => None,
        }
    }

    pub fn to_place(&self) -> Option<Place> {
        match self {
            Operand::Copy(place) => Some(place.clone()),
            Operand::Move(place) => Some(place.clone()),
            Operand::Constant(_) => None,
        }
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Copy(place) => write!(f, "copy {}", place),
            Self::Move(place) => write!(f, "{}", place),
            Self::Constant(constant) => write!(f, "{}", constant),
        }
    }
}
