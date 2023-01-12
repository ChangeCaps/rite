mod local_expr;

pub use local_expr::*;
use ritec_arena::Id;
use ritec_span::Span;

use std::fmt::{self, Display};

use crate::Type;

#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    Local(LocalExpr),
}

impl Display for ExprKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExprKind::Local(expr) => expr.fmt(f),
        }
    }
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

impl Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}): {}", self.kind, self.ty)
    }
}

#[cfg(test)]
mod tests {
    use crate::LocalId;

    use super::*;

    fn local_expr() -> LocalExpr {
        LocalExpr {
            local: LocalId::from_raw_index(0),
        }
    }

    #[test]
    fn display_local_expr() {
        assert_eq!(local_expr().to_string(), "local[0]");
    }

    #[test]
    fn display_expr() {
        let expr = Expr::new(local_expr(), Type::I32);
        assert_eq!(expr.to_string(), "(local[0]): i32");
    }
}
