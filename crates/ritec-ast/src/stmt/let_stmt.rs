use ritec_span::{Ident, Span};

use crate::{Expr, Type};

#[derive(Clone, Debug, PartialEq)]
pub struct LetStmt {
    pub name: Ident,
    pub ty: Option<Type>,
    pub value: Option<Expr>,
    pub span: Span,
}
