use std::hash::{Hash, Hasher};

use ritec_core::Span;

use crate::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IntType {
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FloatType {
    F16,
    F32,
    F64,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PointerType {
    pub pointee: Box<Type>,
}

impl PointerType {
    pub fn new(pointee: impl Into<Box<Type>>) -> Self {
        Self {
            pointee: pointee.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ArrayType {
    pub element: Box<Type>,
    pub size: usize,
}

impl ArrayType {
    pub fn new(element: impl Into<Box<Type>>, size: usize) -> Self {
        Self {
            element: element.into(),
            size,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SliceType {
    pub element: Box<Type>,
}

impl SliceType {
    pub fn new(element: impl Into<Box<Type>>) -> Self {
        Self {
            element: element.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FunctionType {
    pub arguments: Vec<Type>,
    pub return_type: Box<Type>,
}

impl FunctionType {
    pub fn new(arguments: impl Into<Vec<Type>>, return_type: impl Into<Box<Type>>) -> Self {
        Self {
            arguments: arguments.into(),
            return_type: return_type.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TupleType {
    pub elements: Vec<Type>,
}

impl TupleType {
    pub fn new(elements: impl Into<Vec<Type>>) -> Self {
        Self {
            elements: elements.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PathType {
    pub path: Path,
}

impl PathType {
    pub fn new(path: impl Into<Path>) -> Self {
        Self { path: path.into() }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeKind {
    Inferred,
    Void,
    Bool,
    Int(IntType),
    Float(FloatType),
    Pointer(PointerType),
    Array(ArrayType),
    Slice(SliceType),
    Function(FunctionType),
    Tuple(TupleType),
    Path(PathType),
}

#[derive(Clone, Debug)]
pub struct Type {
    pub kind: TypeKind,
    pub span: Span,
}

impl Type {
    pub fn new(kind: impl Into<TypeKind>, span: Span) -> Self {
        Self {
            kind: kind.into(),
            span,
        }
    }

    pub const fn inferred() -> Self {
        Self {
            kind: TypeKind::Inferred,
            span: Span::DUMMY,
        }
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Eq for Type {}

impl Hash for Type {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}
