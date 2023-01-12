mod path_expr;

pub use path_expr::*;

use ritec_span::Span;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Path(PathExpr),
}

impl Expr {
    pub const fn span(&self) -> Span {
        match self {
            Self::Path(expr) => expr.span(),
        }
    }
}
