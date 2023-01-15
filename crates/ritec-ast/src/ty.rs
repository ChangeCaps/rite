use std::{
    fmt::{self, Display},
    hash::Hash,
};

use ritec_core::{FloatSize, IntSize, Span};

use crate::Path;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InferredType {
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VoidType {
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BoolType {
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IntType {
    pub signed: bool,
    pub size: Option<IntSize>,
    pub span: Span,
}

impl Display for IntType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.signed {
            write!(f, "i")?;
        } else {
            write!(f, "u")?;
        }

        if let Some(size) = self.size {
            write!(f, "{}", size.bit_width())?;
        } else {
            write!(f, "size")?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FloatType {
    pub size: FloatSize,
    pub span: Span,
}

impl Display for FloatType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "f{}", self.size.bit_width())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PointerType {
    pub pointee: Box<Type>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ArrayType {
    pub element: Box<Type>,
    pub size: usize,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SliceType {
    pub element: Box<Type>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FunctionType {
    pub arguments: Vec<Type>,
    pub return_type: Box<Type>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TupleType {
    pub fields: Vec<Type>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PathType {
    pub path: Path,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Inferred(InferredType),
    Void(VoidType),
    Bool(BoolType),
    Int(IntType),
    Float(FloatType),
    Pointer(PointerType),
    Array(ArrayType),
    Slice(SliceType),
    Function(FunctionType),
    Tuple(TupleType),
    Path(PathType),
}

impl Type {
    pub const fn span(&self) -> Span {
        match self {
            Type::Inferred(ty) => ty.span,
            Type::Void(ty) => ty.span,
            Type::Bool(ty) => ty.span,
            Type::Int(ty) => ty.span,
            Type::Float(ty) => ty.span,
            Type::Pointer(ty) => ty.span,
            Type::Array(ty) => ty.span,
            Type::Slice(ty) => ty.span,
            Type::Function(ty) => ty.span,
            Type::Tuple(ty) => ty.span,
            Type::Path(ty) => ty.span,
        }
    }
}
