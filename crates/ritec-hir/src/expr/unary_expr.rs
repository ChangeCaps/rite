use ritec_core::Span;

use crate::{ExprId, UniverseId};

#[derive(Clone, Debug, PartialEq)]
pub struct RefExpr {
    pub operand: ExprId,
    pub id: UniverseId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DerefExpr {
    pub operand: ExprId,
    pub id: UniverseId,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Neg,
    Not,
}
