use ritec_core::{BinaryOp, Literal, Span, UnaryOp};

use crate::Path;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Paren(ParenExpr),
    Path(PathExpr),
    Literal(LiteralExpr),
    Call(CallExpr),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Assign(AssignExpr),
    Return(ReturnExpr),
}

impl Expr {
    pub const fn span(&self) -> Span {
        match self {
            Self::Paren(expr) => expr.span,
            Self::Path(expr) => expr.span,
            Self::Literal(expr) => expr.span,
            Self::Call(expr) => expr.span,
            Self::Unary(expr) => expr.span,
            Self::Binary(expr) => expr.span,
            Self::Assign(expr) => expr.span,
            Self::Return(expr) => expr.span,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParenExpr {
    pub expr: Box<Expr>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PathExpr {
    pub path: Path,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LiteralExpr {
    pub literal: Literal,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub arguments: Vec<Expr>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnaryExpr {
    pub operator: UnaryOp,
    pub operand: Box<Expr>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryExpr {
    pub lhs: Box<Expr>,
    pub operator: BinaryOp,
    pub rhs: Box<Expr>,
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
