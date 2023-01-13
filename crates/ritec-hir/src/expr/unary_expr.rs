use ritec_core::Span;

use crate::ExprId;

#[derive(Clone, Debug, PartialEq)]
pub struct RefExpr {
    pub operand: ExprId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DerefExpr {
    pub operand: ExprId,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Neg,
    Not,
}
