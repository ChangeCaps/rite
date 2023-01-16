use ritec_error::{Diagnostic, Emitter};
use ritec_hir as hir;

use crate::{BodyLowerer, Error, RegisteredFunction, Resolver};

pub struct FunctionCompleter<'a> {
    pub program: &'a mut hir::Program,
    pub emitter: &'a mut dyn Emitter,
    pub functions: &'a Vec<RegisteredFunction>,
}

impl<'a> FunctionCompleter<'a> {
    pub fn complete(&mut self) -> Result<(), Error> {
        let mut has_failed = false;

        for function in self.functions.iter() {
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

    pub fn complete_function(&mut self, registered: &RegisteredFunction) -> Result<(), Diagnostic> {
        let mut function = self.program.functions[registered.id].clone();
        let resolver = Resolver {
            program: &self.program,
            generics: &function.generics,
            module: registered.module,
        };

        let mut body_lowerer = BodyLowerer::new(&mut function.body, resolver);
        body_lowerer.lower_block(&registered.block)?;

        self.program.functions[registered.id] = function;

        Ok(())
    }
}
