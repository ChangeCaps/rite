use std::{
    fmt::{self, Display},
    slice::Iter,
    vec::IntoIter,
};

use ritec_core::{Generic, Ident};

#[derive(Clone, Debug, PartialEq)]
pub struct Generics {
    generics: Vec<Generic>,
}

impl Generics {
    pub fn new(generics: impl Into<Vec<Generic>>) -> Self {
        Self {
            generics: generics.into(),
        }
    }

    pub fn get(&self, ident: &Ident) -> Option<&Generic> {
        self.generics.iter().find(|g| g.ident == *ident)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Generic> {
        self.generics.iter()
    }
}

impl From<Vec<Generic>> for Generics {
    fn from(generics: Vec<Generic>) -> Self {
        Self { generics }
    }
}

impl IntoIterator for Generics {
    type Item = Generic;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.generics.into_iter()
    }
}

impl<'a> IntoIterator for &'a Generics {
    type Item = &'a Generic;
    type IntoIter = Iter<'a, Generic>;

    fn into_iter(self) -> Self::IntoIter {
        self.generics.iter()
    }
}

impl Display for Generics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let generics: Vec<_> = self.generics.iter().map(Generic::to_string).collect();
        write!(f, "<{}>", generics.join(", "))
    }
}
