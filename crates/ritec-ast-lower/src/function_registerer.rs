use ritec_ast as ast;
use ritec_core::Generic;
use ritec_error::{Diagnostic, Emitter};
use ritec_hir as hir;

use crate::{Error, Resolver};

pub struct RegisteredFunction {
    pub block: ast::Block,
    pub id: hir::FunctionId,
    pub module: hir::ModuleId,
}

pub struct FunctionRegisterer<'a> {
    pub program: &'a mut hir::Program,
    pub emitter: &'a mut dyn Emitter,
    pub functions: &'a mut Vec<RegisteredFunction>,
    pub module: hir::ModuleId,
}

impl<'a> FunctionRegisterer<'a> {
    pub fn register(&mut self, items: &ast::Items) -> Result<(), Error> {
        let mut has_failed = false;

        for item in items.iter() {
            if let Err(error) = self.register_item(item) {
                self.emitter.emit(error);
                has_failed = true;
            }
        }

        if has_failed {
            Err(Error::FunctionRegistration)
        } else {
            Ok(())
        }
    }

    pub fn register_item(&mut self, item: &ast::Item) -> Result<(), Diagnostic> {
        match item {
            ast::Item::Function(item) => {
                self.register_function(item)?;
            }
        }

        Ok(())
    }

    pub fn register_function(
        &mut self,
        item: &ast::Function,
    ) -> Result<hir::FunctionId, Diagnostic> {
        let mut generic_params = Vec::new();
        for param in item.generics.params.iter() {
            generic_params.push(Generic::new(param.ident.clone()));
        }

        let generics = hir::Generics::new(generic_params, item.generics.span);
        let resolver = Resolver {
            program: self.program,
            generics: &generics,
            module: self.module,
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
                    .with_span("argument type is inferred", argument.span);

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
                .with_span("return type is inferred", item.span);

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

        let function_id = self.program.functions.push(function);

        let registered = RegisteredFunction {
            block: item.body.clone(),
            id: function_id,
            module: self.module,
        };
        self.functions.push(registered);

        self.program[self.module]
            .functions
            .insert(item.ident.clone(), function_id);

        Ok(function_id)
    }
}
