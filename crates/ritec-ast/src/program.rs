use ritec_core::{Arena, Id, Ident};

use crate::{Function, FunctionId};

pub type ModuleId = Id<Module>;

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    pub ident: Ident,
    pub functions: Vec<FunctionId>,
    pub modules: Vec<ModuleId>,
}

impl Module {
    pub fn new(ident: Ident) -> Self {
        Self {
            ident,
            functions: Vec::new(),
            modules: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub root_module: ModuleId,
    pub functions: Arena<Function>,
    pub modules: Arena<Module>,
}

impl Program {
    pub fn new(root_ident: Ident) -> Self {
        let mut modules = Arena::new();
        let root_module = modules.push(Module::new(root_ident));

        Self {
            root_module,
            functions: Arena::new(),
            modules,
        }
    }

    pub fn root(&self) -> &Module {
        &self.modules[self.root_module]
    }

    pub fn root_mut(&mut self) -> &mut Module {
        &mut self.modules[self.root_module]
    }
}
