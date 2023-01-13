use std::collections::HashMap;

use ritec_error::Diagnostic;
use ritec_hir as hir;
use ritec_infer::{InferType, Solver};
use ritec_mir as mir;

pub struct BodyLowerer<'a> {
    pub body: &'a mir::Body,
    pub solver: Solver<'a>,
    pub types: HashMap<hir::UniverseId, InferType>,
}

impl<'a> BodyLowerer<'a> {
    pub fn lower(&mut self, body: &hir::Body) -> Result<(), Diagnostic> {
        Ok(())
    }
}
