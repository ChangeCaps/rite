use ritec_core::{Id, Ident};

use crate::{HirId, Type};

pub type LocalId = Id<Local>;

#[derive(Clone, Debug, PartialEq)]
pub struct Local {
    pub id: HirId,
    pub ident: Ident,
    pub ty: Type,
}
