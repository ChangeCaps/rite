use ritec_core::{Arena, Ident};

use crate::Function;

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    pub name: Ident,
    pub functions: Arena<Function>,
    pub modules: Arena<Module>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Program {
    pub functions: Arena<Function>,
    pub modules: Arena<Module>,
}

impl Program {
    pub fn new() -> Self {
        Self {
            functions: Arena::new(),
            modules: Arena::new(),
        }
    }
}
