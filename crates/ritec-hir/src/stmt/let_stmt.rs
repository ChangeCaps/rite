use ritec_core::Span;

use crate::{ExprId, LocalId, UniverseId};

#[derive(Clone, Debug, PartialEq)]
pub struct LetStmt {
    pub local: LocalId,
    pub init: Option<ExprId>,
    pub id: UniverseId,
    pub span: Span,
}
