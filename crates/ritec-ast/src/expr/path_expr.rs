use ritec_core::Span;

use crate::Path;

#[derive(Clone, Debug, PartialEq)]
pub struct PathExpr {
    pub path: Path,
}

impl PathExpr {
    pub const fn span(&self) -> Span {
        self.path.span
    }
}
