use ritec_span::{Ident, Span};

#[derive(Clone, Debug, PartialEq)]
pub struct GenericParameter {
    pub ident: Ident,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Generics {
    pub params: Vec<GenericParameter>,
    pub span: Span,
}
