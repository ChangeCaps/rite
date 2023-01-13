mod local_expr;
mod unary_expr;

pub use local_expr::*;
pub use unary_expr::*;

use ritec_core::{Id, Span};

use crate::Type;

#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    Local(LocalExpr),
    Unary(UnaryExpr),
}

impl From<LocalExpr> for ExprKind {
    fn from(expr: LocalExpr) -> Self {
        Self::Local(expr)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub ty: Type,
    pub span: Span,
}

pub type ExprId = Id<Expr>;

impl Expr {
    pub fn new(kind: impl Into<ExprKind>, ty: impl Into<Type>) -> Self {
        Self {
            kind: kind.into(),
            ty: ty.into(),
            span: Span::DUMMY,
        }
    }

    pub const fn span(&self) -> Span {
        self.span
    }
}
