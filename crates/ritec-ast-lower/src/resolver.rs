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

    fn get_class(&self, parent: hir::ModuleId, ident: &Ident) -> Result<hir::ClassId, Diagnostic> {
        let module = &self.program[parent];

        if let Some(&class) = module.classes.get(&ident) {
            Ok(class)
        } else {
            let err = Diagnostic::error("class not found")
                .with_msg_span(format!("class '{}' not found", ident), ident.span());

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

    fn resolve_module(&self, segments: &[ast::PathSegment]) -> Result<hir::ModuleId, Diagnostic> {
        let mut module = self.module;

        for segment in segments {
            match segment {
                ast::PathSegment::Item(item) => {
                    self.assert_generic_length(item.generics.len(), 0, item.ident.span())?;

                    module = self.get_module(module, &item.ident)?;
                }
                ast::PathSegment::SuperSegment(span) => {
                    let err = Diagnostic::error("invalid path")
                        .with_msg_span("cannot use `super` in the root module", *span);

                    return Err(err);
                }
                ast::PathSegment::SelfSegment(_) => {}
            }
        }

        Ok(module)
    }

    fn resolve_method(
        &self,
        path: &ast::Path,
    ) -> Result<Option<hir::FunctionInstance>, Diagnostic> {
        let len = path.segments.len();
        if len < 2 {
            return Ok(None);
        }

        let ast::PathSegment::Item(ref class_segment) = path.segments[len - 2] else {
            return Ok(None);
        };
        let ast::PathSegment::Item(ref method_segment) = path.segments[len - 1] else {
            return Ok(None);
        };

        let module = self.resolve_module(&path.segments[..len - 2])?;
        let Ok(class) = self.get_class(module, &class_segment.ident) else {
            return Ok(None);
        };

        let class = &self.program[class];

        if let Some(method) = class.find_method(&method_segment.ident) {
            let mut generics = Vec::new();

            if class_segment.generics.len() == 0 {
                for _ in 0..class.generics.params.len() {
                    generics.push(hir::Type::inferred(class_segment.ident.span()));
                }
            } else {
                for generic in &class_segment.generics {
                    generics.push(self.resolve_type(generic)?);
                }
            }

            let function = &self.program[class[method].function];
            let expected = function.generics.params.len();

            if method_segment.generics.len() == 0 {
                for _ in 0..expected - class.generics.params.len() {
                    generics.push(hir::Type::inferred(method_segment.ident.span()));
                }
            } else {
                for generic in &method_segment.generics {
                    generics.push(self.resolve_type(generic)?);
                }
            }

            self.assert_generic_length(generics.len(), expected, path.span)?;

            let instance = hir::FunctionInstance {
                function: class[method].function,
                generics,
                span: path.span,
            };

            Ok(Some(instance))
        } else {
            Ok(None)
        }
    }

    pub fn resolve_function(
        &self,
        path: &ast::Path,
    ) -> Result<Option<hir::FunctionInstance>, Diagnostic> {
        if let Some(function) = self.resolve_method(path)? {
            return Ok(Some(function));
        }

        Ok(None)
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
        // resolve generics
        if let Some(ident) = ty.path.get_ident() {
            for generic in self.generics.params.iter() {
                if generic.ident == *ident {
                    return Ok(hir::Type::Generic(generic.clone()));
                }
            }
        }

        // resolve class
        let len = ty.path.segments.len();
        let module = self.resolve_module(&ty.path.segments[..len - 1])?;

        let ast::PathSegment::Item(segment) = &ty.path.segments[len - 1] else {
            let err = Diagnostic::error("expected item");    
            return Err(err);
        };

        let Some(&class) = self.program[module].classes.get(&segment.ident) else {
            let err = Diagnostic::error(format!("'{}' not defined", ty.path)).with_span(ty.span);
            return Err(err);
        };

        let expected = self.program[class].generics.params.len();

        let mut generics = Vec::new();
        for generic in &segment.generics {
            generics.push(self.resolve_type(generic)?);
        }

        if generics.len() == 0 {
            for _ in 0..expected {
                generics.push(hir::Type::Inferred(hir::InferredType { span: ty.span }));
            }
        }

        let class_type = hir::ClassType {
            class,
            ident: segment.ident.clone(),
            generics,
            span: ty.span,
        };

        Ok(hir::Type::Class(class_type))
    }
}
