use std::fmt::{self, Display};

use crate::Generic;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VoidType {}

impl VoidType {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Display for VoidType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "void")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BoolType {}

impl BoolType {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Display for BoolType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bool")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IntSize {
    I8 = 1,
    I16 = 2,
    I32 = 4,
    I64 = 8,
    I128 = 16,
}

impl IntSize {
    pub const fn from_byte_size(byte_size: usize) -> Option<Self> {
        match byte_size {
            1 => Some(IntSize::I8),
            2 => Some(IntSize::I16),
            4 => Some(IntSize::I32),
            8 => Some(IntSize::I64),
            16 => Some(IntSize::I128),
            _ => None,
        }
    }

    pub const fn from_bit_width(bit_width: usize) -> Option<Self> {
        match bit_width {
            8 => Some(IntSize::I8),
            16 => Some(IntSize::I16),
            32 => Some(IntSize::I32),
            64 => Some(IntSize::I64),
            128 => Some(IntSize::I128),
            _ => None,
        }
    }

    pub const fn byte_size(self) -> usize {
        self as usize
    }

    pub const fn bit_width(self) -> usize {
        self.byte_size() * 8
    }
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FloatSize {
    F16 = 2,
    F32 = 4,
    F64 = 8,
}

impl FloatSize {
    pub const fn from_byte_size(byte_size: usize) -> Option<Self> {
        match byte_size {
            2 => Some(FloatSize::F16),
            4 => Some(FloatSize::F32),
            8 => Some(FloatSize::F64),
            _ => None,
        }
    }

    pub const fn from_bit_width(bit_width: usize) -> Option<Self> {
        match bit_width {
            16 => Some(FloatSize::F16),
            32 => Some(FloatSize::F32),
            64 => Some(FloatSize::F64),
            _ => None,
        }
    }

    pub const fn byte_size(self) -> usize {
        self as usize
    }

    pub const fn bit_width(self) -> usize {
        self.byte_size() * 8
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
}

impl Display for PointerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "*{}", self.pointee)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ArrayType {
    pub element: Box<Type>,
    pub length: usize,
}

impl ArrayType {
    pub fn new(element: impl Into<Box<Type>>, length: usize) -> Self {
        Self {
            element: element.into(),
            length,
        }
    }
}

impl Display for ArrayType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}; {}]", self.element, self.length)
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
}

impl Display for TupleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields: Vec<_> = self.fields.iter().map(Type::to_string).collect();
        write!(f, "({})", fields.join(", "))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Void(VoidType),
    Bool(BoolType),
    Int(IntType),
    Float(FloatType),
    Pointer(PointerType),
    Array(ArrayType),
    Slice(SliceType),
    Function(FunctionType),
    Tuple(TupleType),
    Generic(Generic),
}

impl Type {
    pub const VOID: Self = Self::Void(VoidType::new());

    pub const BOOL: Self = Self::Bool(BoolType::new());

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
        matches!(self, Type::Void(_))
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
        Self::Function(FunctionType {
            arguments: arguments.into(),
            return_type: return_type.into(),
        })
    }

    pub fn tuple(fields: impl Into<Vec<Type>>) -> Self {
        Self::Tuple(TupleType::new(fields))
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Void(ty) => ty.fmt(f),
            Type::Bool(ty) => ty.fmt(f),
            Type::Int(ty) => ty.fmt(f),
            Type::Float(ty) => ty.fmt(f),
            Type::Pointer(ty) => ty.fmt(f),
            Type::Array(ty) => ty.fmt(f),
            Type::Slice(ty) => ty.fmt(f),
            Type::Function(ty) => ty.fmt(f),
            Type::Tuple(ty) => ty.fmt(f),
            Type::Generic(ty) => ty.fmt(f),
        }
    }
}

impl From<VoidType> for Type {
    fn from(ty: VoidType) -> Self {
        Self::Void(ty)
    }
}

impl From<BoolType> for Type {
    fn from(ty: BoolType) -> Self {
        Self::Bool(ty)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        assert_eq!(Type::VOID.to_string(), "void");
        assert_eq!(Type::BOOL.to_string(), "bool");

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
        assert_eq!(
            Type::function(args.clone(), Type::VOID).to_string(),
            "fn(i32, i64)"
        );

        let args = vec![Type::I32, Type::I64];
        assert_eq!(Type::tuple(args.clone()).to_string(), "(i32, i64)");
    }
}
