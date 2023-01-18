use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use ritec_core::Arena;

use crate::{Class, ClassId, Function, FunctionId};

#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub classes: Arena<Class>,
    pub functions: Arena<Function>,
}

impl Program {
    pub fn new() -> Self {
        Self {
            classes: Arena::new(),
            functions: Arena::new(),
        }
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

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for class in self.classes.values() {
            writeln!(f, "{}\n", class)?;
        }

        for function in self.functions.values() {
            writeln!(f, "{}\n", function)?;
        }

        Ok(())
    }
}
