use ritec_core::Span;

use crate::{ExprId, LocalId, UniverseId};

#[derive(Clone, Debug, PartialEq)]
pub struct LetStmt {
    pub id: UniverseId,
    pub local: LocalId,
    pub init: Option<ExprId>,
    pub span: Span,
}
