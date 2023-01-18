use ritec_ast as ast;
use ritec_core::{Arena, Generic};
use ritec_error::{Diagnostic, Emitter};
use ritec_hir as hir;

use crate::{BodyLowerer, Error, Resolver};

pub struct ProgramLowerer<'a> {
    pub program: &'a mut hir::Program,
    pub emitter: &'a mut dyn Emitter,
}

impl<'a> ProgramLowerer<'a> {
    pub fn new(program: &'a mut hir::Program, emitter: &'a mut dyn Emitter) -> Self {
        Self { program, emitter }
    }

    pub fn lower(&mut self, program: &ast::Program) -> Result<(), Error> {
        self.register_modules(program);
        self.register_classes(program)?;
        self.complete_classes(program)?;
        self.register_functions(program)?;
        self.complete_functions(program)?;

        Ok(())
    }

    pub fn register_modules(&mut self, program: &ast::Program) {
        for (id, module) in program.modules.iter() {
            if !self.program.modules.contains_key(id.cast()) {
                self.program.modules.insert(id.cast(), hir::Module::new());
            }

            let hir = &mut self.program.modules[id.cast()];

            for &id in module.modules.iter() {
                let ident = program.modules[id].ident.clone();
                hir.modules.insert(ident, id.cast());
            }

            for &id in module.classes.iter() {
                let ident = program.classes[id].ident.clone();
                hir.classes.insert(ident, id.cast());
            }

            for &id in module.functions.iter() {
                let ident = program.functions[id].ident.clone();
                hir.functions.insert(ident, id.cast());
            }
        }
    }

    pub fn register_classes(&mut self, program: &ast::Program) -> Result<(), Error> {
        let mut has_failed = false;

        for (id, item) in program.classes.iter() {
            if let Err(err) = self.register_class(id.cast(), item) {
                self.emitter.emit(err.into());
                has_failed = true;
            }
        }

        if has_failed {
            Err(Error::ClassRegistration)
        } else {
            Ok(())
        }
    }

    pub fn register_class(
        &mut self,
        id: hir::ClassId,
        item: &ast::Class,
    ) -> Result<(), Diagnostic> {
        let mut generic_params = Vec::new();
        for param in item.generics.params.iter() {
            generic_params.push(Generic::new(param.ident.clone()));
        }

        let generics = hir::Generics::new(generic_params, item.generics.span);

        let class = hir::Class {
            ident: item.ident.clone(),
            generics,
            fields: Arena::new(),
            span: item.span,
        };

        self.program.classes.insert(id, class);

        Ok(())
    }

    pub fn complete_classes(&mut self, program: &ast::Program) -> Result<(), Error> {
        let mut has_failed = false;

        for (id, item) in program.classes.iter() {
            if let Err(err) = self.complete_class(id.cast(), item) {
                self.emitter.emit(err.into());
                has_failed = true;
            }
        }

        if has_failed {
            Err(Error::ClassCompletion)
        } else {
            Ok(())
        }
    }

    pub fn complete_class(
        &mut self,
        id: hir::ClassId,
        item: &ast::Class,
    ) -> Result<(), Diagnostic> {
        let mut class = self.program[id].clone();
        let resolver = Resolver {
            program: &self.program,
            generics: &class.generics,
            module: item.module.cast(),
        };

        for field in item.fields.iter() {
            let ty = resolver.resolve_type(&field.ty)?;

            let field = hir::Field {
                ident: field.ident.clone(),
                ty,
                span: field.span,
            };

            class.fields.push(field);
        }

        self.program[id] = class;

        Ok(())
    }

    pub fn register_functions(&mut self, program: &ast::Program) -> Result<(), Error> {
        let mut has_failed = false;

        for (id, item) in program.functions.iter() {
            if let Err(err) = self.register_function(id.cast(), item) {
                self.emitter.emit(err.into());
                has_failed = true;
            }
        }

        if has_failed {
            Err(Error::FunctionRegistration)
        } else {
            Ok(())
        }
    }

    pub fn register_function(
        &mut self,
        id: hir::FunctionId,
        item: &ast::Function,
    ) -> Result<(), Diagnostic> {
        let mut generic_params = Vec::new();
        for param in item.generics.params.iter() {
            generic_params.push(Generic::new(param.ident.clone()));
        }

        let generics = hir::Generics::new(generic_params, item.generics.span);
        let resolver = Resolver {
            program: self.program,
            generics: &generics,
            module: item.module.cast(),
        };

        let mut body = hir::Body::new();

        let mut arguments = Vec::new();
        for argument in &item.arguments {
            let ty = resolver.resolve_type(&argument.ty)?;
            let local = hir::Local {
                id: body.next_id(),
                ident: argument.ident.clone(),
                ty: ty.clone(),
            };

            let argument = hir::FunctionArgument {
                ident: argument.ident.clone(),
                local: body.locals.push(local),
                span: argument.span,
            };

            if ty.is_inferred() {
                let err = Diagnostic::error("cannot infer type of function argument")
                    .with_msg_span("argument type is inferred", argument.span);

                return Err(err);
            }

            arguments.push(argument);
        }

        let return_type = if let Some(ty) = &item.return_type {
            resolver.resolve_type(&ty)?
        } else {
            hir::Type::void(item.span)
        };

        if return_type.is_inferred() {
            let err = Diagnostic::error("cannot infer type of function return type")
                .with_msg_span("return type is inferred", item.span);

            return Err(err);
        }

        let function = hir::Function {
            ident: item.ident.clone(),
            generics,
            arguments,
            body,
            return_type,
            span: item.span,
        };

        self.program.functions.insert(id, function);

        Ok(())
    }

    pub fn complete_functions(&mut self, program: &ast::Program) -> Result<(), Error> {
        let mut has_failed = false;

        for (id, function) in program.functions.iter() {
            if let Err(err) = self.complete_function(id.cast(), function) {
                self.emitter.emit(err.into());
                has_failed = true;
            }
        }

        if has_failed {
            Err(Error::FunctionCompletion)
        } else {
            Ok(())
        }
    }

    pub fn complete_function(
        &mut self,
        id: hir::FunctionId,
        item: &ast::Function,
    ) -> Result<(), Diagnostic> {
        let mut function = self.program.functions[id].clone();
        let resolver = Resolver {
            program: &self.program,
            generics: &function.generics,
            module: item.module.cast(),
        };

        let mut body_lowerer = BodyLowerer::new(&mut function.body, resolver);
        body_lowerer.lower_block(&item.body)?;

        self.program.functions[id] = function;

        Ok(())
    }
}
