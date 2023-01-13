use ritec_core::{Id, Ident};

use crate::{Type, UniverseId};

pub type LocalId = Id<Local>;

#[derive(Clone, Debug, PartialEq)]
pub struct Local {
    pub id: UniverseId,
    pub ident: Ident,
    pub ty: Type,
}
