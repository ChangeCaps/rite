use std::fmt::{self, Display};

use crate::{Constant, FloatType, IntType, Operand, Place, PointerType, Type};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    IntNot,
    IntNeg,
    FloatNeg,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BinOp {
    IntAdd,
    IntSub,
    IntMul,
    IntDivSigned,
    IntDivUnsigned,
    IntEq,
    IntNe,
    IntLtSigned,
    IntLtUnsigned,
    IntLeSigned,
    IntLeUnsigned,
    IntGtSigned,
    IntGtUnsigned,
    IntGeSigned,
    IntGeUnsigned,
    FloatAdd,
    FloatSub,
    FloatMul,
    FloatDiv,
    FloatEq,
    FloatNe,
    FloatLt,
    FloatLe,
    FloatGt,
    FloatGe,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Intrinsic {
    Sizeof(Type),
    Alignof(Type),
    Bitcast(Operand, Type),
    Malloc(Operand, Type),
    Free(Operand),
    Memcpy(Operand, Operand, Operand),
    PtrToInt(Operand, PointerType, IntType),
    IntToPtr(Operand, IntType, PointerType),
    PtrToPtr(Operand, PointerType, PointerType),
    IntToInt(Operand, IntType, IntType),
    IntToFloat(Operand, IntType, FloatType),
    FloatToInt(Operand, FloatType, IntType),
    FloatToFloat(Operand, FloatType, FloatType),
}

impl Display for Intrinsic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sizeof(ty) => write!(f, "sizeof({})", ty),
            Self::Alignof(ty) => write!(f, "alignof({})", ty),
            Self::Bitcast(operand, ty) => write!(f, "bitcast({} as {})", operand, ty),
            Self::Malloc(size, ty) => write!(f, "malloc({}, {})", size, ty),
            Self::Free(ptr) => write!(f, "free({})", ptr),
            Self::Memcpy(dst, src, size) => write!(f, "memcpy({}, {}, {})", dst, src, size),
            Self::PtrToInt(operand, from, to) => {
                write!(f, "ptrtoint({} as {} as {})", operand, from, to)
            }
            Self::IntToPtr(operand, from, to) => {
                write!(f, "inttoptr({} as {} as {})", operand, from, to)
            }
            Self::PtrToPtr(operand, from, to) => {
                write!(f, "ptrtoptr({} as {} as {})", operand, from, to)
            }
            Self::IntToInt(operand, from, to) => {
                write!(f, "inttoint({} as {} as {})", operand, from, to)
            }
            Self::IntToFloat(operand, from, to) => {
                write!(f, "inttofloat({} as {} as {})", operand, from, to)
            }
            Self::FloatToInt(operand, from, to) => {
                write!(f, "floattoint({} as {} as {})", operand, from, to)
            }
            Self::FloatToFloat(operand, from, to) => {
                write!(f, "floattofloat({} as {} as {})", operand, from, to)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Use(Operand),
    Address(Place),
    UnaryOp(UnaryOp, Operand),
    BinaryOp(BinOp, Operand, Operand),
    Call(Operand, Vec<Operand>),
    Intrinsic(Intrinsic),
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
            Self::UnaryOp(op, operand) => write!(f, "{:?}({})", op, operand),
            Self::BinaryOp(op, lhs, rhs) => write!(f, "{:?}({}, {})", op, lhs, rhs),
            Self::Call(callee, args) => {
                let args = args.iter().map(Operand::to_string).collect::<Vec<_>>();
                write!(f, "{}({})", callee, args.join(", "))
            }
            Self::Intrinsic(intrinsic) => write!(f, "{}", intrinsic),
        }
    }
}
