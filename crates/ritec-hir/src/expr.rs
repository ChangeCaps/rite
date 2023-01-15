use ritec_core::{BinaryOp, Id, Literal, Span, UnaryOp};

use crate::{FunctionInstance, FunctionType, HirId, LocalId};

pub type ExprId = Id<Expr>;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Local(LocalExpr),
    Literal(LiteralExpr),
    Function(FunctionExpr),
    Call(CallExpr),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Assign(AssignExpr),
    Return(ReturnExpr),
}

impl Expr {
    pub const fn span(&self) -> Span {
        match self {
            Expr::Local(expr) => expr.span,
            Expr::Literal(expr) => expr.span,
            Expr::Function(expr) => expr.span,
            Expr::Call(expr) => expr.span,
            Expr::Unary(expr) => expr.span,
            Expr::Binary(expr) => expr.span,
            Expr::Assign(expr) => expr.span,
            Expr::Return(expr) => expr.span,
        }
    }

    pub const fn id(&self) -> HirId {
        match self {
            Expr::Local(expr) => expr.id,
            Expr::Literal(expr) => expr.id,
            Expr::Function(expr) => expr.id,
            Expr::Call(expr) => expr.id,
            Expr::Unary(expr) => expr.id,
            Expr::Binary(expr) => expr.id,
            Expr::Assign(expr) => expr.id,
            Expr::Return(expr) => expr.id,
        }
    }
}

impl From<LocalExpr> for Expr {
    fn from(expr: LocalExpr) -> Self {
        Self::Local(expr)
    }
}

impl From<LiteralExpr> for Expr {
    fn from(expr: LiteralExpr) -> Self {
        Self::Literal(expr)
    }
}

impl From<FunctionExpr> for Expr {
    fn from(expr: FunctionExpr) -> Self {
        Self::Function(expr)
    }
}

impl From<CallExpr> for Expr {
    fn from(expr: CallExpr) -> Self {
        Self::Call(expr)
    }
}

impl From<UnaryExpr> for Expr {
    fn from(expr: UnaryExpr) -> Self {
        Self::Unary(expr)
    }
}

impl From<BinaryExpr> for Expr {
    fn from(expr: BinaryExpr) -> Self {
        Self::Binary(expr)
    }
}

impl From<AssignExpr> for Expr {
    fn from(expr: AssignExpr) -> Self {
        Self::Assign(expr)
    }
}

impl From<ReturnExpr> for Expr {
    fn from(expr: ReturnExpr) -> Self {
        Self::Return(expr)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalExpr {
    pub local: LocalId,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LiteralExpr {
    pub literal: Literal,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionExpr {
    pub instance: FunctionInstance,
    pub ty: FunctionType,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CallExpr {
    pub callee: ExprId,
    pub arguments: Vec<ExprId>,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnaryExpr {
    pub operator: UnaryOp,
    pub operand: ExprId,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryExpr {
    pub operator: BinaryOp,
    pub lhs: ExprId,
    pub rhs: ExprId,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AssignExpr {
    pub lhs: ExprId,
    pub rhs: ExprId,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReturnExpr {
    pub value: Option<ExprId>,
    pub id: HirId,
    pub span: Span,
}
