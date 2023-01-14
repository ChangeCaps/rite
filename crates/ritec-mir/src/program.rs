use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use ritec_core::Arena;

use crate::{Function, FunctionId};

#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub functions: Arena<Function>,
}

impl Program {
    pub fn new() -> Self {
        Self {
            functions: Arena::new(),
        }
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
        for function in self.functions.values() {
            writeln!(f, "{}", function)?;
        }

        Ok(())
    }
}
