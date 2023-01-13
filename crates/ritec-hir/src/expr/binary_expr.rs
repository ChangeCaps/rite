use ritec_core::Span;

use crate::ExprId;

#[derive(Clone, Debug, PartialEq)]
pub struct AssignExpr {
    pub lhs: ExprId,
    pub rhs: ExprId,
    pub span: Span,
}
