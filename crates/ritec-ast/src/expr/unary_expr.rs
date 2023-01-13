use ritec_core::Span;

use crate::Expr;

#[derive(Clone, Debug, PartialEq)]
pub enum UnaryOp {
    Ref,
    Deref,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnaryExpr {
    pub operator: UnaryOp,
    pub operand: Box<Expr>,
    pub span: Span,
}
