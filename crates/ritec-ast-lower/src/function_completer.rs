use ritec_ast as ast;
use ritec_core::Arena;
use ritec_error::{Diagnostic, Emitter};
use ritec_hir as hir;

use crate::{BodyLowerer, Error, TypeLowerer};

pub struct FunctionCompleter<'a> {
    pub program: &'a mut hir::Program,
    pub emitter: &'a mut dyn Emitter,
    pub blocks: &'a Arena<ast::Block>,
}

impl<'a> FunctionCompleter<'a> {
    pub fn complete(&mut self) -> Result<(), Error> {
        let mut has_failed = false;

        let keys: Vec<_> = self.program.functions.keys().collect();
        for function in keys {
            if let Err(error) = self.complete_function(function) {
                self.emitter.emit(error);
                has_failed = true;
            }
        }

        if has_failed {
            Err(Error::FunctionCompletion)
        } else {
            Ok(())
        }
    }

    pub fn complete_function(&mut self, function_id: hir::FunctionId) -> Result<(), Diagnostic> {
        let mut function = self.program.functions[function_id].clone();
        let type_lowerer = TypeLowerer {
            program: &self.program,
            generics: &function.generics,
            module: function.module,
        };

        let mut body_lowerer = BodyLowerer::new(&mut function.body, type_lowerer);
        body_lowerer.lower_block(&self.blocks[function_id.cast()])?;

        self.program.functions[function_id] = function;

        Ok(())
    }
}
