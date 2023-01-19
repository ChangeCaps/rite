use std::ops::Index;

use ritec_core::{Generic, Ident, Span};

use crate::Type;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Generics {
    pub params: Vec<Generic>,
    pub span: Span,
}

impl Generics {
    pub fn new(params: impl Into<Vec<Generic>>, span: Span) -> Self {
        Self {
            params: params.into(),
            span,
        }
    }

    pub fn empty(span: Span) -> Self {
        Self::new(Vec::new(), span)
    }

    pub fn get_ident(&self, ident: &Ident) -> Option<&Generic> {
        self.params.iter().find(|g| g.ident == *ident)
    }

    pub fn instance(&self) -> Vec<Type> {
        self.params.iter().cloned().map(Type::Generic).collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GenericMap<'a> {
    pub generics: &'a [Generic],
    pub types: &'a [Type],
}

impl<'a> GenericMap<'a> {
    pub fn new(generics: &'a [Generic], types: &'a [Type]) -> Self {
        Self { generics, types }
    }

    pub fn get(&self, ident: &Ident) -> Option<&Type> {
        let index = self.generics.iter().position(|g| g.ident == *ident)?;
        self.types.get(index)
    }
}

impl<'a> Index<&'a Generic> for GenericMap<'a> {
    type Output = Type;

    fn index(&self, index: &'a Generic) -> &Self::Output {
        self.get(index).expect("Generic not found")
    }
}
