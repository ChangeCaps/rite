use std::ops::Index;

use ritec_core::{Arena, Id, Ident, Span};

use crate::{Generics, Type};

pub type FieldId = Id<Field>;

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
    pub fields: Arena<Field>,
    pub span: Span,
}

impl Class {
    pub fn find_field(&self, ident: &Ident) -> Option<FieldId> {
        self.fields
            .iter()
            .rev()
            .find(|(_, field)| field.ident == *ident)
            .map(|(id, _)| id)
    }
}

impl Index<FieldId> for Class {
    type Output = Field;

    fn index(&self, id: FieldId) -> &Self::Output {
        &self.fields[id]
    }
}
