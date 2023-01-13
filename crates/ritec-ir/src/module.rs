use std::collections::BTreeMap;

use ritec_core::{Id, Ident};

use crate::FunctionId;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Module {
    pub modules: BTreeMap<Ident, ModuleId>,
    pub functions: BTreeMap<Ident, FunctionId>,
}

impl Module {
    pub fn new() -> Self {
        Self::default()
    }
}

pub type ModuleId = Id<Module>;
