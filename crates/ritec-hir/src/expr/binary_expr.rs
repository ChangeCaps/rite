use ritec_core::Span;

use crate::{ExprId, UniverseId};

#[derive(Clone, Debug, PartialEq)]
pub struct AssignExpr {
    pub lhs: ExprId,
    pub rhs: ExprId,
    pub id: UniverseId,
    pub span: Span,
}
