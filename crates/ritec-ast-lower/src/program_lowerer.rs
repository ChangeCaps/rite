use std::collections::HashMap;

use ritec_ast as ast;
use ritec_core::SourceMap;
use ritec_error::Emitter;
use ritec_hir as hir;

use crate::{Error, FunctionCompleter, FunctionRegisterer};

pub struct ProgramLowerer<'a> {
    pub program: &'a mut hir::Program,
    pub emitter: &'a mut dyn Emitter,
    pub source_map: &'a mut SourceMap,
}

impl<'a> ProgramLowerer<'a> {
    pub fn lower(&mut self, items: &ast::Items) -> Result<(), Error> {
        let root_module = self.program.root_module;

        let mut modules = HashMap::new();

        let mut functions = Vec::new();
        let mut function_registerer = FunctionRegisterer {
            program: self.program,
            emitter: self.emitter,
            functions: &mut functions,
            module: root_module,
        };
        function_registerer.register(items)?;

        let mut function_completer = FunctionCompleter {
            program: self.program,
            emitter: self.emitter,
            functions: &functions,
        };
        function_completer.complete()?;

        Ok(())
    }

    pub fn register_module(&mut self, module: hir::ModuleId, path: &str) {}
}
