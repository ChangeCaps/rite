mod path_expr;
mod unary_expr;

pub use path_expr::*;
pub use unary_expr::*;

use ritec_core::Span;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Path(PathExpr),
    Unary(UnaryExpr),
}

impl Expr {
    pub const fn span(&self) -> Span {
        match self {
            Self::Path(expr) => expr.span(),
            Self::Unary(expr) => expr.span,
        }
    }
}
