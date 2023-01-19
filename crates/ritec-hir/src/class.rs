use std::ops::Index;

use ritec_core::{Arena, Id, Ident, Span};

use crate::{FunctionId, Generics, Type};

pub type FieldId = Id<Field>;

#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub ident: Ident,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SelfArgument {
    Owned,
    Pointer,
}

pub type MethodId = Id<Method>;

#[derive(Clone, Debug, PartialEq)]
pub struct Method {
    pub ident: Ident,
    pub function: FunctionId,
    pub self_argument: Option<SelfArgument>,
    pub span: Span,
}

pub type ClassId = Id<Class>;

#[derive(Clone, Debug, PartialEq)]
pub struct Class {
    pub ident: Ident,
    pub generics: Generics,
    pub fields: Arena<Field>,
    pub methods: Arena<Method>,
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

    pub fn find_method(&self, ident: &Ident) -> Option<MethodId> {
        self.methods
            .iter()
            .rev()
            .find(|(_, method)| method.ident == *ident)
            .map(|(id, _)| id)
    }
}

impl Index<FieldId> for Class {
    type Output = Field;

    fn index(&self, id: FieldId) -> &Self::Output {
        &self.fields[id]
    }
}

impl Index<MethodId> for Class {
    type Output = Method;

    fn index(&self, id: MethodId) -> &Self::Output {
        &self.methods[id]
    }
}
