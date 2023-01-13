use ritec_core::Span;

use crate::{LocalId, UniverseId};

#[derive(Clone, Debug, PartialEq)]
pub struct LocalExpr {
    pub id: UniverseId,
    pub local: LocalId,
    pub span: Span,
}
