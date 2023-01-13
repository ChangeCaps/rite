mod binary_expr;
mod local_expr;
mod unary_expr;

pub use binary_expr::*;
pub use local_expr::*;
pub use unary_expr::*;

use ritec_core::{Id, Span};

use crate::UniverseId;

pub type ExprId = Id<Expr>;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Local(LocalExpr),
    Ref(RefExpr),
    Deref(DerefExpr),
    Assign(AssignExpr),
}

impl Expr {
    pub const fn span(&self) -> Span {
        match self {
            Expr::Local(expr) => expr.span,
            Expr::Ref(expr) => expr.span,
            Expr::Deref(expr) => expr.span,
            Expr::Assign(expr) => expr.span,
        }
    }

    pub const fn id(&self) -> UniverseId {
        match self {
            Expr::Local(expr) => expr.id,
            Expr::Ref(expr) => expr.id,
            Expr::Deref(expr) => expr.id,
            Expr::Assign(expr) => expr.id,
        }
    }
}
