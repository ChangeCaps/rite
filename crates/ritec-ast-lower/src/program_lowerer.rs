use std::collections::HashMap;

use ritec_ast as ast;
use ritec_core::{Arena, Generic, Ident};
use ritec_error::{Diagnostic, Emitter};
use ritec_hir as hir;

use crate::{BodyLowerer, Error, Resolver};

pub struct ProgramLowerer<'a> {
    pub program: &'a mut hir::Program,
    pub emitter: &'a mut dyn Emitter,
    pub modules: HashMap<ast::ModuleId, hir::ModuleId>,
    pub classes: HashMap<ast::ClassId, hir::ClassId>,
    pub functions: HashMap<ast::FunctionId, hir::FunctionId>,
}

impl<'a> ProgramLowerer<'a> {
    pub fn new(program: &'a mut hir::Program, emitter: &'a mut dyn Emitter) -> Self {
        Self {
            program,
            emitter,
            modules: HashMap::new(),
            classes: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn cast_module(&mut self, ast: ast::ModuleId) -> hir::ModuleId {
        if ast.cast() == self.program.root_module {
            return self.program.root_module;
        }

        if let Some(id) = self.modules.get(&ast) {
            return *id;
        } else {
            let id = self.program.modules.reserve();
            self.modules.insert(ast, id);
            id
        }
    }

    pub fn cast_class(&mut self, ast: ast::ClassId) -> hir::ClassId {
        if let Some(id) = self.classes.get(&ast) {
            return *id;
        } else {
            let id = self.program.classes.reserve();
            self.classes.insert(ast, id);
            id
        }
    }

    pub fn cast_function(&mut self, ast: ast::FunctionId) -> hir::FunctionId {
        if let Some(id) = self.functions.get(&ast) {
            return *id;
        } else {
            let id = self.program.functions.reserve();
            self.functions.insert(ast, id);
            id
        }
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
            let mod_id = self.cast_module(id);

            if !self.program.modules.contains_key(mod_id) {
                self.program.modules.insert(mod_id, hir::Module::new());
            }

            for &id in module.modules.iter() {
                let hir_id = self.cast_module(id);

                let ident = program.modules[id].ident.clone();
                self.program[mod_id].modules.insert(ident, hir_id);
            }

            for &id in module.classes.iter() {
                let hir_id = self.cast_class(id);

                let ident = program.classes[id].ident.clone();
                self.program[mod_id].classes.insert(ident, hir_id);
            }

            for &id in module.functions.iter() {
                let hir_id = self.cast_function(id);

                let ident = program.functions[id].ident.clone();
                self.program[mod_id].functions.insert(ident, hir_id);
            }
        }
    }

    pub fn register_classes(&mut self, program: &ast::Program) -> Result<(), Error> {
        let mut has_failed = false;

        for (id, item) in program.classes.iter() {
            let id = self.cast_class(id);
            if let Err(err) = self.register_class(id, item) {
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
            methods: Arena::new(),
            span: item.span,
        };

        self.program.classes.insert(id, class);

        Ok(())
    }

    pub fn complete_classes(&mut self, program: &ast::Program) -> Result<(), Error> {
        let mut has_failed = false;

        for (id, item) in program.classes.iter() {
            let id = self.cast_class(id);
            if let Err(err) = self.complete_class(id, item) {
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
        let module = self.cast_module(item.module);
        let resolver = Resolver {
            program: &self.program,
            generics: &class.generics,
            module,
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
            let id = self.cast_function(id);
            if let Err(err) = self.register_function(id, item) {
                self.emitter.emit(err.into());
                has_failed = true;
            }
        }

        for (id, item) in program.classes.iter() {
            let id = self.cast_class(id);
            if self.register_methods(id, item).is_err() {
                has_failed = true;
            }
        }

        if has_failed {
            Err(Error::FunctionRegistration)
        } else {
            Ok(())
        }
    }

    pub fn register_methods(&mut self, id: hir::ClassId, class: &ast::Class) -> Result<(), ()> {
        let mut has_failed = false;

        for method in class.methods.iter() {
            let function = self.program.functions.reserve();

            self.program[id].methods.push(hir::Method {
                ident: method.ident.clone(),
                function,
                self_argument: match method.self_argument {
                    Some(ast::SelfArgument::Owned) => Some(hir::SelfArgument::Owned),
                    Some(ast::SelfArgument::Pointer) => Some(hir::SelfArgument::Pointer),
                    None => None,
                },
                span: method.span,
            });

            if let Err(err) = self.register_method(id, function, class, method) {
                self.emitter.emit(err.into());
                has_failed = true;
            }
        }

        if has_failed {
            Err(())
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

        // lower generics
        let generics = hir::Generics::new(generic_params, item.generics.span);
        let module = self.cast_module(item.module);
        let resolver = Resolver {
            program: self.program,
            generics: &generics,
            module,
        };

        let mut body = hir::Body::new();

        // lower arguments
        let mut arguments = Vec::new();

        // lower required arguments
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

            // assert that the argument doesn't contain inferred types
            if ty.is_inferred() {
                let err = Diagnostic::error("cannot infer type of function argument")
                    .with_msg_span("argument type is inferred", argument.span);

                return Err(err);
            }

            arguments.push(argument);
        }

        // lower return type
        let return_type = if let Some(ty) = &item.return_type {
            resolver.resolve_type(&ty)?
        } else {
            hir::Type::void(item.span)
        };

        // assert that the return type doesn't contain inferred types
        if return_type.is_inferred() {
            let err = Diagnostic::error("cannot infer type of function return type")
                .with_msg_span("return type is inferred", item.span);

            return Err(err);
        }

        // create and insert the function
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

    pub fn register_method(
        &mut self,
        class_id: hir::ClassId,
        method_id: hir::FunctionId,
        class: &ast::Class,
        method: &ast::Method,
    ) -> Result<(), Diagnostic> {
        let mut class_generics = Vec::new();
        for param in class.generics.params.iter() {
            class_generics.push(Generic::new(param.ident.clone()));
        }

        let mut method_generics = Vec::new();
        for param in method.generics.params.iter() {
            method_generics.push(Generic::new(param.ident.clone()));
        }

        // lower generics
        let class_generics = hir::Generics::new(class_generics, class.generics.span);
        let method_generics = hir::Generics::new(method_generics, method.generics.span);

        let mut function_generics = class_generics.clone();
        (function_generics.params).extend(method_generics.params.clone());

        let module = self.cast_module(class.module);

        // create resolver
        let resolver = Resolver {
            program: self.program,
            generics: &function_generics,
            module,
        };

        let self_type = hir::Type::Class(hir::ClassType {
            class: class_id,
            ident: class.ident.clone(),
            generics: class_generics.instance(),
            span: class.ident.span(),
        });

        let mut body = hir::Body::new();

        // lower arguments
        let mut arguments = Vec::new();

        // lower self argument
        if let Some(arg) = method.self_argument {
            let local = hir::Local {
                id: body.next_id(),
                ident: Ident::new("self", method.span),
                ty: match arg {
                    ast::SelfArgument::Owned => self_type.clone(),
                    ast::SelfArgument::Pointer => hir::Type::Pointer(hir::PointerType {
                        pointee: Box::new(self_type.clone()),
                        span: method.span,
                    }),
                },
            };

            let argument = hir::FunctionArgument {
                ident: Ident::new("self", method.span),
                local: body.locals.push(local),
                span: method.span,
            };

            arguments.push(argument);
        }

        // lower required arguments
        for argument in &method.arguments {
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

            // assert that the argument doesn't contain inferred types
            if ty.is_inferred() {
                let err = Diagnostic::error("cannot infer type of function argument")
                    .with_msg_span("argument type is inferred", argument.span);

                return Err(err);
            }

            arguments.push(argument);
        }

        // lower return type
        let return_type = if let Some(ty) = &method.return_type {
            resolver.resolve_type(&ty)?
        } else {
            hir::Type::void(method.span)
        };

        // assert that the return type doesn't contain inferred types
        if return_type.is_inferred() {
            let err = Diagnostic::error("cannot infer type of function return type")
                .with_msg_span("return type is inferred", method.span);

            return Err(err);
        }

        // create and insert the function
        let function = hir::Function {
            ident: method.ident.clone(),
            generics: function_generics,
            arguments,
            body,
            return_type,
            span: method.span,
        };

        self.program.functions.insert(method_id, function);

        Ok(())
    }

    pub fn complete_functions(&mut self, program: &ast::Program) -> Result<(), Error> {
        let mut has_failed = false;

        for (id, function) in program.functions.iter() {
            let id = self.cast_function(id);
            if let Err(err) = self.complete_function(id, function.module, &function.body) {
                self.emitter.emit(err.into());
                has_failed = true;
            }
        }

        for (id, class) in program.classes.iter() {
            let id = self.cast_class(id);
            if self.complete_methods(id, class).is_err() {
                has_failed = true;
            }
        }

        if has_failed {
            Err(Error::FunctionCompletion)
        } else {
            Ok(())
        }
    }

    pub fn complete_methods(&mut self, id: hir::ClassId, class: &ast::Class) -> Result<(), ()> {
        let mut has_failed = false;

        for (i, method) in class.methods.iter().enumerate() {
            let function_id = self.program[id].methods[hir::MethodId::from_raw_index(i)].function;

            if let Err(err) = self.complete_function(function_id, class.module, &method.body) {
                self.emitter.emit(err.into());
                has_failed = true;
            }
        }

        if has_failed {
            Err(())
        } else {
            Ok(())
        }
    }

    pub fn complete_function(
        &mut self,
        id: hir::FunctionId,
        module: ast::ModuleId,
        body: &ast::Block,
    ) -> Result<(), Diagnostic> {
        let mut function = self.program.functions[id].clone();
        let module = self.cast_module(module);
        let resolver = Resolver {
            program: &self.program,
            generics: &function.generics,
            module,
        };

        let mut body_lowerer = BodyLowerer::new(&mut function.body, resolver);
        body_lowerer.lower_block(body)?;

        self.program.functions[id] = function;

        Ok(())
    }
}
