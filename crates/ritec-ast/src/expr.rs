use ritec_core::Span;

use crate::Path;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Path(PathExpr),
    Unary(UnaryExpr),
    Assign(AssignExpr),
    Return(ReturnExpr),
}

impl Expr {
    pub const fn span(&self) -> Span {
        match self {
            Self::Path(expr) => expr.span,
            Self::Unary(expr) => expr.span,
            Self::Assign(expr) => expr.span,
            Self::Return(expr) => expr.span,
        }
    }
}

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

#[derive(Clone, Debug, PartialEq)]
pub struct PathExpr {
    pub path: Path,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AssignExpr {
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReturnExpr {
    pub value: Option<Box<Expr>>,
    pub span: Span,
}
