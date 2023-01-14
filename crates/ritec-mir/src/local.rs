use std::fmt::Display;

use ritec_core::{Id, Ident};

use crate::Type;

#[derive(Clone, Debug, PartialEq)]
pub struct Local {
    pub ident: Option<Ident>,
    pub ty: Type,
}

pub type LocalId = Id<Local>;

impl Display for Local {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ": {}", self.ty)?;

        if let Some(ident) = &self.ident {
            write!(f, " // {}", ident)?;
        }

        Ok(())
    }
}
