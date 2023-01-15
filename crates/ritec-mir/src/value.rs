use std::fmt::{self, Display};

use ritec_core::BinaryOp;

use crate::{Constant, Operand, Place};

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryOpValue {
    pub op: BinaryOp,
    pub lhs: Operand,
    pub rhs: Operand,
}

impl Display for BinaryOpValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.lhs, self.op, self.rhs)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Use(Operand),
    Address(Place),
    BinaryOp(BinaryOpValue),
}

impl Value {
    pub const VOID: Self = Self::Use(Operand::Constant(Constant::Void));

    pub fn as_operand(&self) -> Option<&Operand> {
        match self {
            Self::Use(operand) => Some(operand),
            _ => None,
        }
    }

    pub fn to_operand(self) -> Option<Operand> {
        match self {
            Self::Use(operand) => Some(operand),
            _ => None,
        }
    }

    pub fn as_place(&self) -> Option<&Place> {
        self.as_operand()?.as_place()
    }

    pub fn to_place(self) -> Option<Place> {
        self.to_operand()?.to_place()
    }

    pub fn move_operand(place: impl Into<Place>) -> Self {
        Self::Use(Operand::Move(place.into()))
    }
}

impl From<Operand> for Value {
    fn from(operand: Operand) -> Self {
        Self::Use(operand)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Use(operand) => write!(f, "{}", operand),
            Self::Address(place) => write!(f, "&{}", place),
            Self::BinaryOp(value) => write!(f, "{}", value),
        }
    }
}
