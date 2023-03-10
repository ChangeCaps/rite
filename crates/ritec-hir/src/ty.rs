use std::fmt::{self, Display};

use ritec_core::{FloatSize, Generic, Ident, IntSize, Span};

use crate::{ClassId, GenericMap};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InferredType {
    pub span: Span,
}

impl Display for InferredType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "_")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VoidType {
    pub span: Span,
}

impl Display for VoidType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "void")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BoolType {
    pub span: Span,
}

impl Display for BoolType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bool")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IntType {
    pub signed: bool,
    pub size: Option<IntSize>,
    pub span: Span,
}

impl IntType {}

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

impl PointerType {
    pub fn is_inferred(&self) -> bool {
        self.pointee.is_inferred()
    }

    pub fn instantiate(&mut self, generics: &GenericMap<'_>) {
        self.pointee.instantiate(generics);
    }
}

impl Display for PointerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "*{}", self.pointee)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ArrayType {
    pub element: Box<Type>,
    pub size: usize,
    pub span: Span,
}

impl ArrayType {
    pub fn is_inferred(&self) -> bool {
        self.element.is_inferred()
    }

    pub fn instantiate(&mut self, generics: &GenericMap<'_>) {
        self.element.instantiate(generics);
    }
}

impl Display for ArrayType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}; {}]", self.element, self.size)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SliceType {
    pub element: Box<Type>,
    pub span: Span,
}

impl SliceType {
    pub fn is_inferred(&self) -> bool {
        self.element.is_inferred()
    }

    pub fn instantiate(&mut self, generics: &GenericMap<'_>) {
        self.element.instantiate(generics);
    }
}

impl Display for SliceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.element)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FunctionType {
    pub arguments: Vec<Type>,
    pub return_type: Box<Type>,
    pub span: Span,
}

impl FunctionType {
    pub fn is_inferred(&self) -> bool {
        self.arguments.iter().any(|arg| arg.is_inferred()) || self.return_type.is_inferred()
    }

    pub fn instantiate(&mut self, generics: &GenericMap<'_>) {
        for arg in &mut self.arguments {
            arg.instantiate(generics);
        }

        self.return_type.instantiate(generics);
    }
}

impl Display for FunctionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args: Vec<_> = self.arguments.iter().map(Type::to_string).collect();
        write!(f, "fn({})", args.join(", "))?;

        if !self.return_type.is_void() {
            write!(f, " -> {}", self.return_type)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TupleType {
    pub fields: Vec<Type>,
    pub span: Span,
}

impl TupleType {
    pub fn is_inferred(&self) -> bool {
        self.fields.iter().any(|field| field.is_inferred())
    }

    pub fn instantiate(&mut self, generics: &GenericMap<'_>) {
        for field in &mut self.fields {
            field.instantiate(generics);
        }
    }
}

impl Display for TupleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields: Vec<_> = self.fields.iter().map(Type::to_string).collect();
        write!(f, "({})", fields.join(", "))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ClassType {
    pub class: ClassId,
    pub ident: Ident,
    pub generics: Vec<Type>,
    pub span: Span,
}

impl ClassType {
    pub fn is_inferred(&self) -> bool {
        self.generics.iter().any(|generic| generic.is_inferred())
    }

    pub fn instantiate(&mut self, generics: &GenericMap<'_>) {
        for generic in &mut self.generics {
            generic.instantiate(generics);
        }
    }
}

impl Display for ClassType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let generics: Vec<_> = self.generics.iter().map(Type::to_string).collect();
        write!(f, "{}<{}>", self.ident, generics.join(", "))
    }
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
    Class(ClassType),
    Generic(Generic),
}

impl Type {
    pub const fn span(&self) -> Span {
        match self {
            Type::Inferred(t) => t.span,
            Type::Void(t) => t.span,
            Type::Bool(t) => t.span,
            Type::Int(t) => t.span,
            Type::Float(t) => t.span,
            Type::Pointer(t) => t.span,
            Type::Array(t) => t.span,
            Type::Slice(t) => t.span,
            Type::Function(t) => t.span,
            Type::Tuple(t) => t.span,
            Type::Class(t) => t.span,
            Type::Generic(t) => t.span(),
        }
    }

    pub const fn is_void(&self) -> bool {
        matches!(self, Type::Void(_))
    }

    pub const fn inferred(span: Span) -> Self {
        Type::Inferred(InferredType { span })
    }

    pub const fn void(span: Span) -> Self {
        Type::Void(VoidType { span })
    }

    pub const fn bool(span: Span) -> Self {
        Type::Bool(BoolType { span })
    }

    pub fn is_inferred(&self) -> bool {
        match self {
            Type::Inferred(_) => true,
            Type::Pointer(t) => t.is_inferred(),
            Type::Array(t) => t.is_inferred(),
            Type::Slice(t) => t.is_inferred(),
            Type::Function(t) => t.is_inferred(),
            Type::Tuple(t) => t.is_inferred(),
            _ => false,
        }
    }

    pub fn instantiate(&mut self, generics: &GenericMap<'_>) {
        match self {
            Type::Pointer(t) => t.instantiate(generics),
            Type::Array(t) => t.instantiate(generics),
            Type::Slice(t) => t.instantiate(generics),
            Type::Function(t) => t.instantiate(generics),
            Type::Tuple(t) => t.instantiate(generics),
            Type::Class(t) => t.instantiate(generics),
            Type::Generic(generic) => *self = generics[generic].clone(),
            Type::Inferred(_) | Type::Void(_) | Type::Bool(_) | Type::Int(_) | Type::Float(_) => {}
        }
    }
}

impl From<InferredType> for Type {
    fn from(t: InferredType) -> Self {
        Type::Inferred(t)
    }
}

impl From<VoidType> for Type {
    fn from(t: VoidType) -> Self {
        Type::Void(t)
    }
}

impl From<BoolType> for Type {
    fn from(t: BoolType) -> Self {
        Type::Bool(t)
    }
}

impl From<IntType> for Type {
    fn from(t: IntType) -> Self {
        Type::Int(t)
    }
}

impl From<FloatType> for Type {
    fn from(t: FloatType) -> Self {
        Type::Float(t)
    }
}

impl From<PointerType> for Type {
    fn from(t: PointerType) -> Self {
        Type::Pointer(t)
    }
}

impl From<ArrayType> for Type {
    fn from(t: ArrayType) -> Self {
        Type::Array(t)
    }
}

impl From<SliceType> for Type {
    fn from(t: SliceType) -> Self {
        Type::Slice(t)
    }
}

impl From<FunctionType> for Type {
    fn from(t: FunctionType) -> Self {
        Type::Function(t)
    }
}

impl From<TupleType> for Type {
    fn from(t: TupleType) -> Self {
        Type::Tuple(t)
    }
}

impl From<ClassType> for Type {
    fn from(t: ClassType) -> Self {
        Type::Class(t)
    }
}

impl From<Generic> for Type {
    fn from(t: Generic) -> Self {
        Type::Generic(t)
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Inferred(t) => write!(f, "{}", t),
            Type::Void(t) => write!(f, "{}", t),
            Type::Bool(t) => write!(f, "{}", t),
            Type::Int(t) => write!(f, "{}", t),
            Type::Float(t) => write!(f, "{}", t),
            Type::Pointer(t) => write!(f, "{}", t),
            Type::Array(t) => write!(f, "{}", t),
            Type::Slice(t) => write!(f, "{}", t),
            Type::Function(t) => write!(f, "{}", t),
            Type::Tuple(t) => write!(f, "{}", t),
            Type::Class(t) => write!(f, "{}", t),
            Type::Generic(t) => write!(f, "{}", t),
        }
    }
}
