mod binary_expr;
mod local_expr;
mod unary_expr;

pub use binary_expr::*;
pub use local_expr::*;
pub use unary_expr::*;

use ritec_core::{Id, Span};

use crate::UniverseId;

#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    Local(LocalExpr),
    Ref(RefExpr),
    Deref(DerefExpr),
    Assign(AssignExpr),
}

impl ExprKind {
    pub const fn span(&self) -> Span {
        match self {
            ExprKind::Local(expr) => expr.span,
            ExprKind::Ref(expr) => expr.span,
            ExprKind::Deref(expr) => expr.span,
            ExprKind::Assign(expr) => expr.span,
        }
    }
}

pub type ExprId = Id<Expr>;

#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    pub id: UniverseId,
    pub kind: ExprKind,
}

impl Expr {
    pub const fn span(&self) -> Span {
        self.kind.span()
    }
}
