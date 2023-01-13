use ritec_ast as ast;
use ritec_core::{Arena, Generic};
use ritec_error::{Diagnostic, Emitter};
use ritec_hir as hir;

use crate::{Error, TypeLowerer};

pub struct FunctionRegisterer<'a> {
    pub program: &'a mut hir::Program,
    pub emitter: &'a mut dyn Emitter,
    pub blocks: &'a mut Arena<ast::Block>,
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
        item: &ast::FunctionItem,
    ) -> Result<hir::FunctionId, Diagnostic> {
        let mut generic_params = Vec::new();
        for param in item.generics.params.iter() {
            generic_params.push(Generic::new(param.ident.clone()));
        }

        let generics = hir::Generics::new(generic_params, item.generics.span);
        let type_lowerer = TypeLowerer {
            program: self.program,
            generics: &generics,
            module: self.module,
        };

        let mut body = hir::Body::new();

        let mut arguments = Vec::new();
        for argument in &item.arguments {
            let local = hir::Local {
                id: body.next_universe_id(),
                ident: argument.ident.clone(),
                ty: type_lowerer.lower_type(&argument.ty)?,
            };

            let argument = hir::FunctionArgument {
                ident: argument.ident.clone(),
                ty: type_lowerer.lower_type(&argument.ty)?,
                local: body.locals.push(local),
                span: argument.span,
            };

            if argument.ty.is_inferred() {
                let err = Diagnostic::error("cannot infer type of function argument")
                    .with_message_span("argument type is inferred", argument.span);

                return Err(err);
            }

            arguments.push(argument);
        }

        let return_type = if let Some(ty) = &item.return_type {
            type_lowerer.lower_type(&ty)?
        } else {
            hir::Type::void(item.span)
        };

        if return_type.is_inferred() {
            let err = Diagnostic::error("cannot infer type of function return type")
                .with_message_span("return type is inferred", item.span);

            return Err(err);
        }

        let function = hir::Function {
            ident: item.ident.clone(),
            module: self.module,
            generics,
            arguments,
            body,
            return_type,
            span: item.span,
        };

        let function_id = self.program.functions.push(function);
        self.blocks.insert(function_id.cast(), item.body.clone());

        self.program[self.module]
            .functions
            .insert(item.ident.clone(), function_id);

        Ok(function_id)
    }
}
