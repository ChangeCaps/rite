use ritec_core::{Id, Ident};

use crate::Type;

#[derive(Clone, Debug, PartialEq)]
pub struct Local {
    pub ident: Ident,
    pub ty: Type,
}

pub type LocalId = Id<Local>;
