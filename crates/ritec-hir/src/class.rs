use ritec_core::{Id, Ident, Span};

use crate::{Generics, Type};

#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub ident: Ident,
    pub ty: Type,
    pub span: Span,
}

pub type ClassId = Id<Class>;

#[derive(Clone, Debug, PartialEq)]
pub struct Class {
    pub ident: Ident,
    pub generics: Generics,
    pub fields: Vec<Field>,
    pub span: Span,
}
