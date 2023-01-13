use ritec_core::{Ident, Span};

use crate::{Block, Generics, Type};

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionArgument {
    pub ident: Ident,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionItem {
    pub ident: Ident,
    pub generics: Generics,
    pub arguments: Vec<FunctionArgument>,
    pub return_type: Option<Type>,
    pub body: Block,
    pub span: Span,
}
