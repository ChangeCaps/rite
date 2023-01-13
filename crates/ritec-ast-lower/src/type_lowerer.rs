use ritec_ast as ast;
use ritec_error::Diagnostic;
use ritec_hir::{self as hir, Generics};

pub struct TypeLowerer<'a> {
    pub program: &'a hir::Program,
    pub generics: &'a Generics,
    pub module: hir::ModuleId,
}

impl<'a> TypeLowerer<'a> {
    pub fn lower_type(&self, ty: &ast::Type) -> Result<hir::Type, Diagnostic> {
        match ty {
            ast::Type::Inferred(ty) => Ok(self.lower_inferred_type(ty)),
            ast::Type::Void(ty) => Ok(self.lower_void_type(ty)),
            ast::Type::Bool(ty) => Ok(self.lower_bool_type(ty)),
            ast::Type::Int(ty) => Ok(self.lower_int_type(ty)),
            ast::Type::Float(ty) => Ok(self.lower_float_type(ty)),
            ast::Type::Pointer(ty) => self.lower_pointer_type(ty),
            ast::Type::Array(ty) => self.lower_array_type(ty),
            ast::Type::Slice(ty) => self.lower_slice_type(ty),
            ast::Type::Function(ty) => self.lower_function_type(ty),
            ast::Type::Tuple(ty) => self.lower_tuple_type(ty),
            ast::Type::Path(ty) => self.lower_path_type(ty),
        }
    }

    pub fn lower_inferred_type(&self, ty: &ast::InferredType) -> hir::Type {
        let inferred_type = hir::InferredType { span: ty.span };

        hir::Type::Inferred(inferred_type)
    }

    pub fn lower_void_type(&self, ty: &ast::VoidType) -> hir::Type {
        let void_type = hir::VoidType { span: ty.span };

        hir::Type::Void(void_type)
    }

    pub fn lower_bool_type(&self, ty: &ast::BoolType) -> hir::Type {
        let bool_type = hir::BoolType { span: ty.span };

        hir::Type::Bool(bool_type)
    }

    pub fn lower_int_type(&self, ty: &ast::IntType) -> hir::Type {
        let int_type = hir::IntType {
            signed: ty.signed,
            size: ty.size,
            span: ty.span,
        };

        hir::Type::Int(int_type)
    }

    pub fn lower_float_type(&self, ty: &ast::FloatType) -> hir::Type {
        let float_type = hir::FloatType {
            size: ty.size,
            span: ty.span,
        };

        hir::Type::Float(float_type)
    }

    pub fn lower_pointer_type(&self, ty: &ast::PointerType) -> Result<hir::Type, Diagnostic> {
        let pointer_type = hir::PointerType {
            pointee: Box::new(self.lower_type(&ty.pointee)?),
            span: ty.span,
        };

        Ok(hir::Type::Pointer(pointer_type))
    }

    pub fn lower_array_type(&self, ty: &ast::ArrayType) -> Result<hir::Type, Diagnostic> {
        let array_type = hir::ArrayType {
            element: Box::new(self.lower_type(&ty.element)?),
            size: ty.size,
            span: ty.span,
        };

        Ok(hir::Type::Array(array_type))
    }

    pub fn lower_slice_type(&self, ty: &ast::SliceType) -> Result<hir::Type, Diagnostic> {
        let slice_type = hir::SliceType {
            element: Box::new(self.lower_type(&ty.element)?),
            span: ty.span,
        };

        Ok(hir::Type::Slice(slice_type))
    }

    pub fn lower_function_type(&self, ty: &ast::FunctionType) -> Result<hir::Type, Diagnostic> {
        let mut arguments = Vec::new();
        for argument in &ty.arguments {
            arguments.push(self.lower_type(argument)?);
        }

        let function_type = hir::FunctionType {
            arguments,
            return_type: Box::new(self.lower_type(&ty.return_type)?),
            span: ty.span,
        };

        Ok(hir::Type::Function(function_type))
    }

    pub fn lower_tuple_type(&self, ty: &ast::TupleType) -> Result<hir::Type, Diagnostic> {
        let mut fields = Vec::new();
        for field in &ty.fields {
            fields.push(self.lower_type(field)?);
        }

        let tuple_type = hir::TupleType {
            fields,
            span: ty.span,
        };

        Ok(hir::Type::Tuple(tuple_type))
    }

    pub fn lower_path_type(&self, ty: &ast::PathType) -> Result<hir::Type, Diagnostic> {
        if let Some(ident) = ty.path.get_ident() {
            if let Some(generic) = self.generics.get_ident(ident) {
                let generic_type = hir::GenericType {
                    generic: generic.clone(),
                    span: ty.span,
                };

                return Ok(hir::Type::Generic(generic_type));
            }

            todo!()
        } else {
            todo!()
        }
    }
}
