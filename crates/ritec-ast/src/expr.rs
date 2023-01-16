use ritec_core::{BinOp, Literal, Span, UnaryOp};

use crate::{Block, Path};

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
    Break(BreakExpr),
    Block(BlockExpr),
    If(IfExpr),
    Loop(LoopExpr),
    While(WhileExpr),
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
            Self::Break(expr) => expr.span,
            Self::Block(expr) => expr.span,
            Self::If(expr) => expr.span,
            Self::Loop(expr) => expr.span,
            Self::While(expr) => expr.span,
        }
    }

    pub const fn stmt_needs_semi(&self) -> bool {
        match self {
            Self::Block(_) | Self::If(_) | Self::Loop(_) | Self::While(_) => false,
            _ => true,
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
    pub operator: BinOp,
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

#[derive(Clone, Debug, PartialEq)]
pub struct BreakExpr {
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BlockExpr {
    pub block: Block,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfExpr {
    pub condition: Box<Expr>,
    pub then_block: Box<Expr>,
    pub else_block: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoopExpr {
    pub block: Block,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WhileExpr {
    pub condition: Box<Expr>,
    pub block: Block,
    pub span: Span,
}
