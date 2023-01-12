use ritec_ast as ast;

use ritec_ir::{Body, Function, FunctionArgument, FunctionId, Local, ModuleId, Program, Type};

use crate::{LowerError, TypeLowerer};

pub struct ProgramLowerer<'a> {
    pub program: &'a mut Program,
    pub module: ModuleId,
}

impl<'a> ProgramLowerer<'a> {
    pub fn new(program: &'a mut Program) -> Self {
        Self {
            module: program.root_module(),
            program,
        }
    }

    pub fn type_lowerer(&mut self, generics: &ast::Generics) -> TypeLowerer<'_> {
        TypeLowerer::new(self.program, generics, self.module)
    }

    pub fn lower_items(&mut self, items: &ast::Items) -> Result<(), LowerError> {
        for item in items.items.iter() {
            match item {
                ast::Item::Function(item) => {
                    self.lower_function(item)?;
                }
            }
        }

        Ok(())
    }

    pub fn lower_function(
        &mut self,
        function: &ast::FunctionItem,
    ) -> Result<FunctionId, LowerError> {
        let mut body = Body::new();

        let type_lowerer = self.type_lowerer(&function.generics);

        let mut arguments = Vec::with_capacity(function.arguments.len());
        for argument in function.arguments.iter() {
            let ident = argument.ident.clone();
            let ty = type_lowerer.lower_type(&argument.ty)?;

            let local = Local {
                ident: ident.clone(),
                ty: ty.clone(),
            };

            let local = body.locals.push(local);
            let argument = FunctionArgument { ident, ty, local };
            arguments.push(argument);
        }

        let return_type = if let Some(ref return_type) = function.return_type {
            type_lowerer.lower_type(return_type)?
        } else {
            Type::VOID
        };

        let function = Function {
            ident: function.ident.clone(),
            generics: type_lowerer.generics,
            arguments,
            return_type,
            body,
        };

        Ok(self.program.functions.push(function))
    }
}
