use ritec_ast as ast;
use ritec_core::Arena;
use ritec_error::Emitter;
use ritec_hir as hir;

use crate::{FunctionCompleter, FunctionRegisterer, LowerError};

pub struct ProgramLowerer<'a> {
    pub program: &'a mut hir::Program,
    pub emitter: &'a mut dyn Emitter,
}

impl<'a> ProgramLowerer<'a> {
    pub fn new(program: &'a mut hir::Program, emitter: &'a mut dyn Emitter) -> Self {
        Self { program, emitter }
    }

    pub fn lower(&mut self, items: &ast::Items) -> Result<(), LowerError> {
        let root_module = self.program.root_module;

        let mut blocks = Arena::new();
        let mut function_registerer = FunctionRegisterer {
            program: self.program,
            emitter: self.emitter,
            blocks: &mut blocks,
            module: root_module,
        };
        function_registerer.register(items)?;

        let mut function_completer = FunctionCompleter {
            program: self.program,
            emitter: self.emitter,
            blocks: &blocks,
        };
        function_completer.complete()?;

        Ok(())
    }
}
