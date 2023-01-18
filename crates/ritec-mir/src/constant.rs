use std::fmt::{self, Display};

use crate::{FloatType, FunctionId, IntType, Type};

#[derive(Clone, Debug, PartialEq)]
pub enum Constant {
    Void,
    Null,
    Function(FunctionId, Vec<Type>),
    Integer(i64, IntType),
    Float(f64, FloatType),
    Bool(bool),
}

impl Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Void => write!(f, "void"),
            Self::Null => write!(f, "null"),
            Self::Function(id, generics) => {
                let generics: Vec<_> = generics.iter().map(Type::to_string).collect();
                write!(f, "fn[{}]<{}>", id.as_raw_index(), generics.join(", "))
            }
            Self::Integer(c, ty) => write!(f, "{}{}", c, ty),
            Self::Float(c, ty) => write!(f, "{}{}", c, ty),
            Self::Bool(c) => write!(f, "{}", c),
        }
    }
}
