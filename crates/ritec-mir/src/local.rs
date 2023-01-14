use ritec_core::{Id, Ident};

use crate::Type;

pub type LocalId = Id<Local>;

#[derive(Clone, Debug, PartialEq)]
pub struct Local {
    pub ident: Option<Ident>,
    pub ty: Type,
}

impl Local {
    pub const fn new(ty: Type) -> Self {
        Self { ident: None, ty }
    }

    pub const fn with_ident(ident: Ident, ty: Type) -> Self {
        Self {
            ident: Some(ident),
            ty,
        }
    }

    pub fn comment(&self) -> String {
        match &self.ident {
            Some(ident) => format!("// {}", ident),
            None => String::new(),
        }
    }
}
