use std::fmt::{self, Display};

use crate::Span;

#[derive(Clone, Debug, PartialEq)]
pub struct BoolLiteral {
    pub value: bool,
    pub span: Span,
}

impl Display for BoolLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum IntPrefix {
    #[default]
    Dec,
    Hex,
    Oct,
    Bin,
}

impl IntPrefix {
    pub const fn radix(self) -> u32 {
        match self {
            Self::Dec => 10,
            Self::Hex => 16,
            Self::Oct => 8,
            Self::Bin => 2,
        }
    }
}

impl Display for IntPrefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dec => write!(f, ""),
            Self::Hex => write!(f, "0x"),
            Self::Oct => write!(f, "0o"),
            Self::Bin => write!(f, "0b"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct IntLiteral {
    pub prefix: IntPrefix,
    pub value: u64,
    pub span: Span,
}

impl Display for IntLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.prefix, self.value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FloatLiteral {
    pub value: f64,
    pub span: Span,
}

impl Display for FloatLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Bool(BoolLiteral),
    Int(IntLiteral),
    Float(FloatLiteral),
}

impl Literal {
    pub const fn span(&self) -> Span {
        match self {
            Self::Bool(lit) => lit.span,
            Self::Int(lit) => lit.span,
            Self::Float(lit) => lit.span,
        }
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(lit) => write!(f, "{}", lit),
            Self::Int(lit) => write!(f, "{}", lit),
            Self::Float(lit) => write!(f, "{}", lit),
        }
    }
}
