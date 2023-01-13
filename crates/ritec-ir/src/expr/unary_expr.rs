use crate::ExprId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Ref,
    Deref,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UnaryExpr {
    pub operator: UnaryOp,
    pub operand: ExprId,
}
