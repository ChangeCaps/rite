use ritec_core::{Id, Span};

use crate::{HirId, LocalId};

pub type ExprId = Id<Expr>;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Local(LocalExpr),
    Ref(RefExpr),
    Deref(DerefExpr),
    Assign(AssignExpr),
    Return(ReturnExpr),
}

impl Expr {
    pub const fn span(&self) -> Span {
        match self {
            Expr::Local(expr) => expr.span,
            Expr::Ref(expr) => expr.span,
            Expr::Deref(expr) => expr.span,
            Expr::Assign(expr) => expr.span,
            Expr::Return(expr) => expr.span,
        }
    }

    pub const fn id(&self) -> HirId {
        match self {
            Expr::Local(expr) => expr.id,
            Expr::Ref(expr) => expr.id,
            Expr::Deref(expr) => expr.id,
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

impl From<RefExpr> for Expr {
    fn from(expr: RefExpr) -> Self {
        Self::Ref(expr)
    }
}

impl From<DerefExpr> for Expr {
    fn from(expr: DerefExpr) -> Self {
        Self::Deref(expr)
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
pub struct RefExpr {
    pub operand: ExprId,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DerefExpr {
    pub operand: ExprId,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Neg,
    Not,
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
