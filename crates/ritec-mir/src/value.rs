use std::fmt::{self, Display};

use crate::{Constant, Operand, Place};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BinOp {
    IntAdd,
    IntSub,
    IntMul,
    IntDivSigned,
    IntDivUnsigned,
    FloatAdd,
    FloatSub,
    FloatMul,
    FloatDiv,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Use(Operand),
    Address(Place),
    BinaryOp(BinOp, Operand, Operand),
    Call(Operand, Vec<Operand>),
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
            Self::BinaryOp(op, lhs, rhs) => write!(f, "{:?}({}, {})", op, lhs, rhs),
            Self::Call(callee, args) => {
                let args = args.iter().map(Operand::to_string).collect::<Vec<_>>();
                write!(f, "{}({})", callee, args.join(", "))
            }
        }
    }
}
