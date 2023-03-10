use std::fmt::{self, Display};

use ritec_core::{FloatSize, Generic, Ident, IntSize};

use crate::{ClassId, GenericMap};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IntType {
    pub signed: bool,
    pub size: Option<IntSize>,
}

impl IntType {
    pub const fn new(signed: bool, size: Option<IntSize>) -> Self {
        Self { signed, size }
    }

    pub const fn byte_size(&self) -> Option<usize> {
        if let Some(size) = self.size {
            Some(size.byte_size())
        } else {
            None
        }
    }

    pub const fn bit_width(&self) -> Option<usize> {
        if let Some(size) = self.size {
            Some(size.bit_width())
        } else {
            None
        }
    }

    pub const fn is_signed(&self) -> bool {
        self.signed
    }

    pub const fn is_unsigned(&self) -> bool {
        !self.signed
    }

    pub const fn is_pointer_size(&self) -> bool {
        self.size.is_none()
    }

    pub const I8: Self = Self::new(true, Some(IntSize::I8));
    pub const I16: Self = Self::new(true, Some(IntSize::I16));
    pub const I32: Self = Self::new(true, Some(IntSize::I32));
    pub const I64: Self = Self::new(true, Some(IntSize::I64));
    pub const I128: Self = Self::new(true, Some(IntSize::I128));

    pub const U8: Self = Self::new(false, Some(IntSize::I8));
    pub const U16: Self = Self::new(false, Some(IntSize::I16));
    pub const U32: Self = Self::new(false, Some(IntSize::I32));
    pub const U64: Self = Self::new(false, Some(IntSize::I64));
    pub const U128: Self = Self::new(false, Some(IntSize::I128));

    pub const ISIZE: Self = Self::new(true, None);
    pub const USIZE: Self = Self::new(false, None);
}

impl Display for IntType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // write sign prefix
        if self.signed {
            write!(f, "i")?;
        } else {
            write!(f, "u")?;
        }

        // write bit width
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
}

impl FloatType {
    pub const fn new(size: FloatSize) -> Self {
        Self { size }
    }

    pub const fn byte_size(&self) -> usize {
        self.size.byte_size()
    }

    pub const fn bit_width(&self) -> usize {
        self.size.bit_width()
    }

    pub const F16: Self = Self::new(FloatSize::F16);
    pub const F32: Self = Self::new(FloatSize::F32);
    pub const F64: Self = Self::new(FloatSize::F64);
}

impl Display for FloatType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "f{}", self.size.bit_width())
    }
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

    pub const fn pointee(&self) -> &Type {
        &self.pointee
    }

    pub fn instantiate(&mut self, generics: &GenericMap) {
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
}

impl ArrayType {
    pub fn new(element: impl Into<Box<Type>>, length: usize) -> Self {
        Self {
            element: element.into(),
            size: length,
        }
    }

    pub fn instantiate(&mut self, generics: &GenericMap) {
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
}

impl SliceType {
    pub fn new(element: impl Into<Box<Type>>) -> Self {
        Self {
            element: element.into(),
        }
    }

    pub fn instantiate(&mut self, generics: &GenericMap) {
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
}

impl FunctionType {
    pub fn new(arguments: impl Into<Vec<Type>>, return_type: impl Into<Box<Type>>) -> Self {
        Self {
            arguments: arguments.into(),
            return_type: return_type.into(),
        }
    }

    pub fn instantiate(&mut self, generics: &GenericMap) {
        for argument in &mut self.arguments {
            argument.instantiate(generics);
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
}

impl TupleType {
    pub fn new(fields: impl Into<Vec<Type>>) -> Self {
        Self {
            fields: fields.into(),
        }
    }

    pub fn instantiate(&mut self, generics: &GenericMap) {
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
}

impl ClassType {
    pub fn new(class: ClassId, ident: Ident, generics: impl Into<Vec<Type>>) -> Self {
        Self {
            class,
            ident,
            generics: generics.into(),
        }
    }

    pub fn instantiate(&mut self, generics: &GenericMap) {
        for generic in &mut self.generics {
            generic.instantiate(generics);
        }
    }
}

impl Display for ClassType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.generics.is_empty() {
            write!(f, "{}", self.ident)
        } else {
            let generics: Vec<_> = self.generics.iter().map(Type::to_string).collect();
            write!(f, "{}<{}>", self.ident, generics.join(", "))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Void,
    Bool,
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
    pub const I8: Self = Self::Int(IntType::I8);
    pub const I16: Self = Self::Int(IntType::I16);
    pub const I32: Self = Self::Int(IntType::I32);
    pub const I64: Self = Self::Int(IntType::I64);
    pub const I128: Self = Self::Int(IntType::I128);

    pub const U8: Self = Self::Int(IntType::U8);
    pub const U16: Self = Self::Int(IntType::U16);
    pub const U32: Self = Self::Int(IntType::U32);
    pub const U64: Self = Self::Int(IntType::U64);
    pub const U128: Self = Self::Int(IntType::U128);

    pub const ISIZE: Self = Self::Int(IntType::ISIZE);
    pub const USIZE: Self = Self::Int(IntType::USIZE);

    pub const F16: Self = Self::Float(FloatType::F16);
    pub const F32: Self = Self::Float(FloatType::F32);
    pub const F64: Self = Self::Float(FloatType::F64);

    pub const fn is_void(&self) -> bool {
        matches!(self, Type::Void)
    }

    pub const fn is_pointer(&self) -> bool {
        matches!(self, Type::Pointer(_))
    }

    pub fn pointer(pointee: impl Into<Box<Type>>) -> Self {
        Self::Pointer(PointerType::new(pointee))
    }

    pub fn array(element: impl Into<Box<Type>>, length: usize) -> Self {
        Self::Array(ArrayType::new(element, length))
    }

    pub fn slice(element: impl Into<Box<Type>>) -> Self {
        Self::Slice(SliceType::new(element))
    }

    pub fn function(arguments: impl Into<Vec<Type>>, return_type: impl Into<Box<Type>>) -> Self {
        Self::Function(FunctionType::new(arguments, return_type))
    }

    pub fn tuple(fields: impl Into<Vec<Type>>) -> Self {
        Self::Tuple(TupleType::new(fields))
    }

    pub fn class(class: ClassId, ident: Ident, generics: impl Into<Vec<Type>>) -> Self {
        Self::Class(ClassType::new(class, ident, generics))
    }

    pub fn deref(&self) -> &Type {
        match self {
            Type::Pointer(pointer) => pointer.pointee.deref(),
            _ => self,
        }
    }

    pub fn instantiate(&mut self, generics: &GenericMap) {
        match self {
            Type::Void => {}
            Type::Bool => {}
            Type::Int(_) => {}
            Type::Float(_) => {}
            Type::Pointer(pointer) => pointer.instantiate(generics),
            Type::Array(array) => array.instantiate(generics),
            Type::Slice(slice) => slice.instantiate(generics),
            Type::Function(function) => function.instantiate(generics),
            Type::Tuple(tuple) => tuple.instantiate(generics),
            Type::Class(class) => class.instantiate(generics),
            Type::Generic(generic) => {
                if let Some(replacement) = generics.get(generic) {
                    *self = replacement.clone();
                }
            }
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Void => write!(f, "void"),
            Type::Bool => write!(f, "bool"),
            Type::Int(ty) => ty.fmt(f),
            Type::Float(ty) => ty.fmt(f),
            Type::Pointer(ty) => ty.fmt(f),
            Type::Array(ty) => ty.fmt(f),
            Type::Slice(ty) => ty.fmt(f),
            Type::Function(ty) => ty.fmt(f),
            Type::Tuple(ty) => ty.fmt(f),
            Type::Class(ty) => ty.fmt(f),
            Type::Generic(ty) => ty.fmt(f),
        }
    }
}

impl From<IntType> for Type {
    fn from(ty: IntType) -> Self {
        Self::Int(ty)
    }
}

impl From<FloatType> for Type {
    fn from(ty: FloatType) -> Self {
        Self::Float(ty)
    }
}

impl From<PointerType> for Type {
    fn from(ty: PointerType) -> Self {
        Self::Pointer(ty)
    }
}

impl From<ArrayType> for Type {
    fn from(ty: ArrayType) -> Self {
        Self::Array(ty)
    }
}

impl From<SliceType> for Type {
    fn from(ty: SliceType) -> Self {
        Self::Slice(ty)
    }
}

impl From<FunctionType> for Type {
    fn from(ty: FunctionType) -> Self {
        Self::Function(ty)
    }
}

impl From<TupleType> for Type {
    fn from(ty: TupleType) -> Self {
        Self::Tuple(ty)
    }
}

impl From<ClassType> for Type {
    fn from(ty: ClassType) -> Self {
        Self::Class(ty)
    }
}

impl From<Generic> for Type {
    fn from(ty: Generic) -> Self {
        Self::Generic(ty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        assert_eq!(Type::Void.to_string(), "void");
        assert_eq!(Type::Bool.to_string(), "bool");

        assert_eq!(Type::I8.to_string(), "i8");
        assert_eq!(Type::I16.to_string(), "i16");
        assert_eq!(Type::I32.to_string(), "i32");
        assert_eq!(Type::I64.to_string(), "i64");
        assert_eq!(Type::I128.to_string(), "i128");

        assert_eq!(Type::U8.to_string(), "u8");
        assert_eq!(Type::U16.to_string(), "u16");
        assert_eq!(Type::U32.to_string(), "u32");
        assert_eq!(Type::U64.to_string(), "u64");
        assert_eq!(Type::U128.to_string(), "u128");

        assert_eq!(Type::ISIZE.to_string(), "isize");
        assert_eq!(Type::USIZE.to_string(), "usize");

        assert_eq!(Type::F16.to_string(), "f16");
        assert_eq!(Type::F32.to_string(), "f32");
        assert_eq!(Type::F64.to_string(), "f64");

        assert_eq!(Type::pointer(Type::I32).to_string(), "*i32");
        assert_eq!(Type::array(Type::I32, 10).to_string(), "[i32; 10]");
        assert_eq!(Type::slice(Type::I32).to_string(), "[i32]");

        let args = vec![Type::I32, Type::I64];
        assert_eq!(
            Type::function(args.clone(), Type::I128).to_string(),
            "fn(i32, i64) -> i128"
        );

        let args = vec![Type::I32, Type::I64];
        assert_eq!(Type::tuple(args.clone()).to_string(), "(i32, i64)");
    }
}
