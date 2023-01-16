use std::ops::Index;

use ritec_core::{Generic, Ident};

use crate::Type;

#[derive(Clone, Copy)]
pub struct GenericMap<'a> {
    generics: &'a [Generic],
    types: &'a [Type],
}

impl<'a> GenericMap<'a> {
    pub fn new(generics: &'a [Generic], types: &'a [Type]) -> Self {
        Self { generics, types }
    }

    pub fn get(&self, ident: &Ident) -> Option<&Type> {
        self.generics
            .iter()
            .position(|g| g.ident == *ident)
            .map(|i| &self.types[i])
    }
}

impl<'a> Index<&'a Generic> for GenericMap<'a> {
    type Output = Type;

    fn index(&self, index: &'a Generic) -> &Self::Output {
        self.get(&index.ident).unwrap()
    }
}
