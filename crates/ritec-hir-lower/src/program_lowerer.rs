use ritec_error::{Diagnostic, Emitter};
use ritec_hir as hir;
use ritec_infer::Solver;
use ritec_mir as mir;

use crate::{BodyLowerer, Error};

pub struct ProgramLowerer<'a> {
    pub program: &'a mut mir::Program,
    pub emitter: &'a mut dyn Emitter,
}

impl<'a> ProgramLowerer<'a> {
    pub fn new(program: &'a mut mir::Program, emitter: &'a mut dyn Emitter) -> Self {
        Self { program, emitter }
    }

    pub fn lower_type(&self, ty: &hir::Type) -> mir::Type {
        match ty {
            hir::Type::Inferred(_) => unreachable!("inferred type in illegal position"),
            hir::Type::Void(ty) => mir::Type::Void(self.lower_void_type(ty)),
            hir::Type::Bool(ty) => mir::Type::Bool(self.lower_bool_type(ty)),
            hir::Type::Int(ty) => mir::Type::Int(self.lower_int_type(ty)),
            hir::Type::Float(ty) => mir::Type::Float(self.lower_float_type(ty)),
            hir::Type::Pointer(ty) => mir::Type::Pointer(self.lower_pointer_type(ty)),
            hir::Type::Array(ty) => mir::Type::Array(self.lower_array_type(ty)),
            hir::Type::Slice(ty) => mir::Type::Slice(self.lower_slice_type(ty)),
            hir::Type::Function(ty) => mir::Type::Function(self.lower_function_type(ty)),
            hir::Type::Tuple(ty) => mir::Type::Tuple(self.lower_tuple_type(ty)),
            hir::Type::Generic(ty) => mir::Type::Generic(self.lower_generic_type(ty)),
        }
    }

    pub fn lower_void_type(&self, _: &hir::VoidType) -> mir::VoidType {
        mir::VoidType
    }

    pub fn lower_bool_type(&self, _: &hir::BoolType) -> mir::BoolType {
        mir::BoolType
    }

    pub fn lower_int_type(&self, ty: &hir::IntType) -> mir::IntType {
        mir::IntType {
            signed: ty.signed,
            size: ty.size,
        }
    }

    pub fn lower_float_type(&self, ty: &hir::FloatType) -> mir::FloatType {
        mir::FloatType { size: ty.size }
    }

    pub fn lower_pointer_type(&self, ty: &hir::PointerType) -> mir::PointerType {
        mir::PointerType {
            pointee: Box::new(self.lower_type(&ty.pointee)),
        }
    }

    pub fn lower_array_type(&self, ty: &hir::ArrayType) -> mir::ArrayType {
        mir::ArrayType {
            element: Box::new(self.lower_type(&ty.element)),
            size: ty.size,
        }
    }

    pub fn lower_slice_type(&self, ty: &hir::SliceType) -> mir::SliceType {
        mir::SliceType {
            element: Box::new(self.lower_type(&ty.element)),
        }
    }

    pub fn lower_function_type(&self, ty: &hir::FunctionType) -> mir::FunctionType {
        let mut arguments = Vec::with_capacity(ty.arguments.len());
        for argument in &ty.arguments {
            arguments.push(self.lower_type(argument));
        }

        mir::FunctionType {
            arguments,
            return_type: Box::new(self.lower_type(&ty.return_type)),
        }
    }

    pub fn lower_tuple_type(&self, ty: &hir::TupleType) -> mir::TupleType {
        let mut fields = Vec::with_capacity(ty.fields.len());
        for field in &ty.fields {
            fields.push(self.lower_type(field));
        }

        mir::TupleType { fields }
    }

    pub fn lower_generic_type(&self, ty: &hir::GenericType) -> mir::GenericType {
        mir::GenericType {
            generic: ty.generic.clone(),
        }
    }

    pub fn lower(&mut self, program: &hir::Program) -> Result<(), Error> {
        self.lower_functions(program)?;

        Ok(())
    }

    pub fn lower_functions(&mut self, program: &hir::Program) -> Result<(), Error> {
        let mut has_failed = false;

        for (id, function) in program.functions.iter() {
            if let Err(diagnostic) = self.lower_function(id, function) {
                self.emitter.emit(diagnostic);

                has_failed = true;
            }
        }

        if has_failed {
            Err(Error::FunctionLowering)
        } else {
            Ok(())
        }
    }

    pub fn lower_function(
        &mut self,
        id: hir::FunctionId,
        function: &hir::Function,
    ) -> Result<(), Diagnostic> {
        let mut generics = Vec::with_capacity(function.generics.params.len());
        for generic in function.generics.params.iter() {
            generics.push(generic.clone());
        }

        let mut arguments = Vec::with_capacity(function.arguments.len());
        for argument in function.arguments.iter() {
            let argument = mir::FunctionArgument {
                ident: argument.ident.clone(),
                ty: self.lower_type(&argument.ty),
                local: argument.local.cast(),
            };

            arguments.push(argument);
        }

        let return_type = self.lower_type(&function.return_type);

        let mut body = mir::Body::new();

        let mut body_lowerer = BodyLowerer::new(
            &function.body,
            &mut body,
            &function.return_type,
            Solver::new(&self.program),
        );

        body_lowerer.infer()?;
        body_lowerer.lower()?;

        let function = mir::Function {
            ident: function.ident.clone(),
            generics: mir::Generics::new(generics),
            arguments,
            return_type,
            body,
        };

        self.program.functions.insert(id.cast(), function);

        Ok(())
    }
}
