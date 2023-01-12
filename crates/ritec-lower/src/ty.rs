use ritec_ast as ast;
use ritec_ir::{Generic, Generics, ModuleId, Program, Type};

use crate::LowerError;

pub struct TypeLowerer<'a> {
    pub program: &'a Program,
    pub generics: Generics,
    pub module: ModuleId,
}

impl<'a> TypeLowerer<'a> {
    pub fn new(program: &'a Program, ast_generics: &ast::Generics, module: ModuleId) -> Self {
        let mut generics = Vec::new();
        for param in ast_generics.params.iter() {
            let generic = Generic::new(param.ident.clone());
            generics.push(generic);
        }

        Self {
            program,
            generics: Generics::new(generics),
            module,
        }
    }

    pub fn lower_type(&self, ty: &ast::Type) -> Result<Type, LowerError> {
        match ty.kind {
            ast::TypeKind::Inferred => Err(LowerError::InvalidInferred),
            ast::TypeKind::Void => Ok(Type::VOID),
            ast::TypeKind::Bool => Ok(Type::BOOL),
            ast::TypeKind::Int(ty) => match ty {
                ast::IntType::I8 => Ok(Type::I8),
                ast::IntType::I16 => Ok(Type::I16),
                ast::IntType::I32 => Ok(Type::I32),
                ast::IntType::I64 => Ok(Type::I64),
                ast::IntType::I128 => Ok(Type::I128),
                ast::IntType::Isize => Ok(Type::ISIZE),
                ast::IntType::U8 => Ok(Type::U8),
                ast::IntType::U16 => Ok(Type::U16),
                ast::IntType::U32 => Ok(Type::U32),
                ast::IntType::U64 => Ok(Type::U64),
                ast::IntType::U128 => Ok(Type::U128),
                ast::IntType::Usize => Ok(Type::USIZE),
            },
            ast::TypeKind::Float(ty) => match ty {
                ast::FloatType::F16 => Ok(Type::F16),
                ast::FloatType::F32 => Ok(Type::F32),
                ast::FloatType::F64 => Ok(Type::F64),
            },
            ast::TypeKind::Pointer(ref ty) => self.lower_pointer_type(ty),
            ast::TypeKind::Array(ref ty) => self.lower_array_type(ty),
            ast::TypeKind::Slice(ref ty) => self.lower_slice_type(ty),
            ast::TypeKind::Function(ref ty) => self.lower_function_type(ty),
            ast::TypeKind::Tuple(ref ty) => self.lower_tuple_type(ty),
            ast::TypeKind::Path(ref ty) => self.lower_path_type(ty),
        }
    }

    pub fn lower_pointer_type(&self, ty: &ast::PointerType) -> Result<Type, LowerError> {
        let pointee = self.lower_type(&ty.pointee)?;
        Ok(Type::pointer(pointee))
    }

    pub fn lower_array_type(&self, ty: &ast::ArrayType) -> Result<Type, LowerError> {
        let element = self.lower_type(&ty.element)?;
        Ok(Type::array(element, ty.size))
    }

    pub fn lower_slice_type(&self, ty: &ast::SliceType) -> Result<Type, LowerError> {
        let element = self.lower_type(&ty.element)?;
        Ok(Type::slice(element))
    }

    pub fn lower_function_type(&self, ty: &ast::FunctionType) -> Result<Type, LowerError> {
        let mut params = Vec::new();
        for param in ty.arguments.iter() {
            params.push(self.lower_type(param)?);
        }

        let ret = self.lower_type(&ty.return_type)?;

        Ok(Type::function(params, ret))
    }

    pub fn lower_tuple_type(&self, ty: &ast::TupleType) -> Result<Type, LowerError> {
        let mut elements = Vec::new();
        for element in ty.elements.iter() {
            elements.push(self.lower_type(element)?);
        }

        Ok(Type::tuple(elements))
    }

    pub fn lower_path_type(&self, ty: &ast::PathType) -> Result<Type, LowerError> {
        if let Some(ident) = ty.path.get_ident() {
            if let Some(generic) = self.generics.get(ident) {
                return Ok(Type::Generic(generic.clone()));
            }
        }

        Err(LowerError::UndefinedType(ty.path.clone()))
    }
}
