use ritec_core::Span;

use crate::{LocalId, UniverseId};

#[derive(Clone, Debug, PartialEq)]
pub struct LocalExpr {
    pub local: LocalId,
    pub id: UniverseId,
    pub span: Span,
}
