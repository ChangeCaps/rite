use ritec_core::{BinaryOp, Literal, Span, UnaryOp};

use crate::Path;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Paren(Box<Expr>),
    Path(PathExpr),
    Literal(LiteralExpr),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Assign(AssignExpr),
    Return(ReturnExpr),
}

impl Expr {
    pub const fn span(&self) -> Span {
        match self {
            Expr::Paren(expr) => expr.span(),
            Self::Path(expr) => expr.span,
            Self::Literal(expr) => expr.span,
            Self::Unary(expr) => expr.span,
            Self::Binary(expr) => expr.span,
            Self::Assign(expr) => expr.span,
            Self::Return(expr) => expr.span,
        }
    }
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

impl BinaryExpr {
    pub fn operator(&self) -> &BinaryOp {
        &self.operator
    }
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
