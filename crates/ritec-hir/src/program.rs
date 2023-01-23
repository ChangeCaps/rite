use std::ops::{Index, IndexMut};

use ritec_core::Arena;

use crate::{
    build_intrinsic_alignof, build_intrinsic_bitcast, build_intrinsic_free, build_intrinsic_malloc,
    build_intrinsic_memcpy, build_intrinsic_sizeof, Class, ClassId, Function, FunctionId, Module,
    ModuleId,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub root_module: ModuleId,
    pub auto_include: ModuleId,
    pub modules: Arena<Module>,
    pub classes: Arena<Class>,
    pub functions: Arena<Function>,
}

impl Program {
    pub fn new() -> Self {
        let mut modules = Arena::new();
        let classes = Arena::new();
        let functions = Arena::new();

        let root_module = modules.push(Module::new());
        let auto_include = modules.push(Module::new());

        Self {
            root_module,
            auto_include,
            modules,
            classes,
            functions,
        }
    }

    pub fn add_function(&mut self, function: Function) -> FunctionId {
        let ident = function.ident.clone();
        let id = self.functions.push(function);
        self.modules[self.auto_include].functions.insert(ident, id);
        id
    }

    pub fn add_intrinsics(&mut self) {
        self.add_function(build_intrinsic_bitcast());
        self.add_function(build_intrinsic_sizeof());
        self.add_function(build_intrinsic_alignof());
        self.add_function(build_intrinsic_malloc());
        self.add_function(build_intrinsic_free());
        self.add_function(build_intrinsic_memcpy());
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

impl Index<ClassId> for Program {
    type Output = Class;

    fn index(&self, index: ClassId) -> &Self::Output {
        &self.classes[index]
    }
}

impl IndexMut<ClassId> for Program {
    fn index_mut(&mut self, index: ClassId) -> &mut Self::Output {
        &mut self.classes[index]
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
