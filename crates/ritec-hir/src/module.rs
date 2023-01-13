use std::collections::BTreeMap;

use ritec_core::{Id, Ident};

use crate::FunctionId;

pub type ModuleId = Id<Module>;

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    pub modules: BTreeMap<Ident, ModuleId>,
    pub functions: BTreeMap<Ident, FunctionId>,
}

impl Module {
    pub fn new() -> Self {
        Self {
            modules: BTreeMap::new(),
            functions: BTreeMap::new(),
        }
    }
}
