use ritec_ast as ast;
use ritec_core::SourceMap;
use ritec_error::Emitter;
use ritec_hir as hir;

use crate::Error;

pub struct ModuleRegisterer<'a> {
    pub program: &'a mut hir::Program,
    pub emitter: &'a mut dyn Emitter,
    pub source_map: &'a mut SourceMap,
    pub module: hir::ModuleId,
}

impl<'a> ModuleRegisterer<'a> {
    pub fn register(&mut self, items: &ast::Items) -> Result<(), Error> {
        for item in items.iter() {
            match item {
                ast::Item::Module(item) => {}
                _ => {}
            }
        }

        Ok(())
    }
}
