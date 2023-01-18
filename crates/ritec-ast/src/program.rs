use ritec_core::{Arena, Id, Ident};

use crate::{Class, ClassId, Function, FunctionId};

pub type ModuleId = Id<Module>;

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    pub ident: Ident,
    pub modules: Vec<ModuleId>,
    pub classes: Vec<ClassId>,
    pub functions: Vec<FunctionId>,
}

impl Module {
    pub fn new(ident: Ident) -> Self {
        Self {
            ident,
            modules: Vec::new(),
            classes: Vec::new(),
            functions: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub root_module: ModuleId,
    pub modules: Arena<Module>,
    pub classes: Arena<Class>,
    pub functions: Arena<Function>,
}

impl Program {
    pub fn new(root_ident: Ident) -> Self {
        let mut modules = Arena::new();
        let root_module = modules.push(Module::new(root_ident));

        Self {
            root_module,
            modules,
            classes: Arena::new(),
            functions: Arena::new(),
        }
    }

    pub fn root(&self) -> &Module {
        &self.modules[self.root_module]
    }

    pub fn root_mut(&mut self) -> &mut Module {
        &mut self.modules[self.root_module]
    }
}
