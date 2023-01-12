use ritec_span::{Ident, Span};

use crate::{Expr, Type};

#[derive(Clone, Debug, PartialEq)]
pub struct LetStmt {
    pub ident: Ident,
    pub ty: Option<Type>,
    pub init: Option<Expr>,
    pub span: Span,
}
