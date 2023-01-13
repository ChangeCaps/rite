use std::ops::{Index, IndexMut};

use ritec_core::Arena;

use crate::{Function, FunctionId, Module, ModuleId};

#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub root_module: ModuleId,
    pub modules: Arena<Module>,
    pub functions: Arena<Function>,
}

impl Program {
    pub fn new() -> Self {
        let mut modules = Arena::new();
        let functions = Arena::new();

        let root_module = modules.push(Module::new());

        Self {
            root_module,
            modules,
            functions,
        }
    }
}

impl Index<ModuleId> for Program {
    type Output = Module;

    fn index(&self, index: ModuleId) -> &Self::Output {
        &self.modules[index]
    }
}

impl IndexMut<ModuleId> for Program {
    fn index_mut(&mut self, index: ModuleId) -> &mut Self::Output {
        &mut self.modules[index]
    }
}

impl Index<FunctionId> for Program {
    type Output = Function;

    fn index(&self, index: FunctionId) -> &Self::Output {
        &self.functions[index]
    }
}

impl IndexMut<FunctionId> for Program {
    fn index_mut(&mut self, index: FunctionId) -> &mut Self::Output {
        &mut self.functions[index]
    }
}