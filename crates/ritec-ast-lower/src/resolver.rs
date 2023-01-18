use ritec_ast as ast;
use ritec_core::{Ident, Span};
use ritec_error::Diagnostic;
use ritec_hir::{self as hir, Generics};

pub struct Resolver<'a> {
    pub program: &'a hir::Program,
    pub generics: &'a Generics,
    pub module: hir::ModuleId,
}

impl<'a> Resolver<'a> {
    fn get_module(
        &self,
        parent: hir::ModuleId,
        ident: &Ident,
    ) -> Result<hir::ModuleId, Diagnostic> {
        let module = &self.program[parent];

        if let Some(&module) = module.modules.get(&ident) {
            Ok(module)
        } else {
            let err = Diagnostic::error("module not found")
                .with_msg_span(format!("module '{}' not found", ident), ident.span());

            Err(err)
        }
    }

    fn assert_generic_length(
        &self,
        actual: usize,
        expected: usize,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if actual != expected {
            let err = Diagnostic::error("invalid number of generic arguments").with_msg_span(
                format!("expected {} generic arguments, found {}", expected, actual),
                span,
            );

            Err(err)
        } else {
            Ok(())
        }
    }

    pub fn resolve_function(
        &self,
        path: &ast::Path,
    ) -> Result<Option<hir::FunctionInstance>, Diagnostic> {
        let mut module_id = self.module;

        let mut iter = path.segments.iter().peekable();
        loop {
            let segment = iter.next().unwrap();

            match segment {
                ast::PathSegment::Item(item) => {
                    let module = &self.program[module_id];

                    if iter.peek().is_some() {
                        module_id = self.get_module(module_id, &item.ident)?;
                        continue;
                    }

                    let Some(&function) = module.functions.get(&item.ident) else {
                        return Ok(None);
                    };

                    let expected_len = self.program[function].generics.params.len();

                    let mut generics = Vec::new();
                    for generic in item.generics.iter() {
                        generics.push(self.resolve_type(generic)?);
                    }

                    if generics.len() == 0 {
                        for _ in 0..expected_len {
                            generics.push(hir::Type::inferred(path.span));
                        }
                    }

                    self.assert_generic_length(generics.len(), expected_len, path.span)?;

                    return Ok(Some(hir::FunctionInstance {
                        function,
                        generics,
                        span: path.span,
                    }));
                }
                ast::PathSegment::SuperSegment(_) => todo!(),
                ast::PathSegment::SelfSegment(_) => todo!(),
            }
        }
    }

    pub fn resolve_type(&self, ty: &ast::Type) -> Result<hir::Type, Diagnostic> {
        match ty {
            ast::Type::Inferred(ty) => Ok(self.resolve_inferred_type(ty)),
            ast::Type::Void(ty) => Ok(self.resolve_void_type(ty)),
            ast::Type::Bool(ty) => Ok(self.resolve_bool_type(ty)),
            ast::Type::Int(ty) => Ok(self.resolve_int_type(ty)),
            ast::Type::Float(ty) => Ok(self.resolve_float_type(ty)),
            ast::Type::Pointer(ty) => self.resolve_pointer_type(ty),
            ast::Type::Array(ty) => self.resolver_array_type(ty),
            ast::Type::Slice(ty) => self.resolve_slice_type(ty),
            ast::Type::Function(ty) => self.resolve_function_type(ty),
            ast::Type::Tuple(ty) => self.resolve_tuple_type(ty),
            ast::Type::Path(ty) => self.resolve_path_type(ty),
        }
    }

    pub fn resolve_inferred_type(&self, ty: &ast::InferredType) -> hir::Type {
        let inferred_type = hir::InferredType { span: ty.span };

        hir::Type::Inferred(inferred_type)
    }

    pub fn resolve_void_type(&self, ty: &ast::VoidType) -> hir::Type {
        let void_type = hir::VoidType { span: ty.span };

        hir::Type::Void(void_type)
    }

    pub fn resolve_bool_type(&self, ty: &ast::BoolType) -> hir::Type {
        let bool_type = hir::BoolType { span: ty.span };

        hir::Type::Bool(bool_type)
    }

    pub fn resolve_int_type(&self, ty: &ast::IntType) -> hir::Type {
        let int_type = hir::IntType {
            signed: ty.signed,
            size: ty.size,
            span: ty.span,
        };

        hir::Type::Int(int_type)
    }

    pub fn resolve_float_type(&self, ty: &ast::FloatType) -> hir::Type {
        let float_type = hir::FloatType {
            size: ty.size,
            span: ty.span,
        };

        hir::Type::Float(float_type)
    }

    pub fn resolve_pointer_type(&self, ty: &ast::PointerType) -> Result<hir::Type, Diagnostic> {
        let pointer_type = hir::PointerType {
            pointee: Box::new(self.resolve_type(&ty.pointee)?),
            span: ty.span,
        };

        Ok(hir::Type::Pointer(pointer_type))
    }

    pub fn resolver_array_type(&self, ty: &ast::ArrayType) -> Result<hir::Type, Diagnostic> {
        let array_type = hir::ArrayType {
            element: Box::new(self.resolve_type(&ty.element)?),
            size: ty.size,
            span: ty.span,
        };

        Ok(hir::Type::Array(array_type))
    }

    pub fn resolve_slice_type(&self, ty: &ast::SliceType) -> Result<hir::Type, Diagnostic> {
        let slice_type = hir::SliceType {
            element: Box::new(self.resolve_type(&ty.element)?),
            span: ty.span,
        };

        Ok(hir::Type::Slice(slice_type))
    }

    pub fn resolve_function_type(&self, ty: &ast::FunctionType) -> Result<hir::Type, Diagnostic> {
        let mut arguments = Vec::new();
        for argument in &ty.arguments {
            arguments.push(self.resolve_type(argument)?);
        }

        let function_type = hir::FunctionType {
            arguments,
            return_type: Box::new(self.resolve_type(&ty.return_type)?),
            span: ty.span,
        };

        Ok(hir::Type::Function(function_type))
    }

    pub fn resolve_tuple_type(&self, ty: &ast::TupleType) -> Result<hir::Type, Diagnostic> {
        let mut fields = Vec::new();
        for field in &ty.fields {
            fields.push(self.resolve_type(field)?);
        }

        let tuple_type = hir::TupleType {
            fields,
            span: ty.span,
        };

        Ok(hir::Type::Tuple(tuple_type))
    }

    pub fn resolve_path_type(&self, ty: &ast::PathType) -> Result<hir::Type, Diagnostic> {
        if let Some(ident) = ty.path.get_ident() {
            if let Some(generic) = self.generics.get_ident(ident) {
                return Ok(hir::Type::Generic(generic.clone()));
            }
        }

        let mut module_id = self.module;

        let mut iter = ty.path.segments.iter().peekable();

        loop {
            let segment = iter.next().unwrap();

            match segment {
                ast::PathSegment::Item(item) => {
                    let module = &self.program[module_id];

                    if iter.peek().is_some() {
                        module_id = self.get_module(module_id, &item.ident)?;
                        continue;
                    }

                    let Some(&class) = module.classes.get(&item.ident) else {
                        break;
                    };

                    let expected_len = self.program[class].generics.params.len();

                    let mut generics = Vec::new();
                    for generic in &item.generics {
                        generics.push(self.resolve_type(generic)?);
                    }

                    if generics.len() == 0 {
                        for _ in 0..expected_len {
                            generics.push(hir::Type::Inferred(hir::InferredType { span: ty.span }));
                        }
                    }

                    self.assert_generic_length(expected_len, generics.len(), ty.span)?;

                    let class_type = hir::ClassType {
                        class,
                        ident: item.ident.clone(),
                        generics,
                        span: ty.span,
                    };

                    return Ok(hir::Type::Class(class_type));
                }
                ast::PathSegment::SuperSegment(_) => todo!(),
                ast::PathSegment::SelfSegment(_) => todo!(),
            }
        }

        let err = Diagnostic::error(format!("'{}' not defined", ty.path)).with_span(ty.span);

        Err(err)
    }
}
