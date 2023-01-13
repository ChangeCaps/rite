use std::{
    fmt::{self, Display},
    hash::{Hash, Hasher},
    ops::Deref,
    slice::Iter,
    sync::atomic::{AtomicUsize, Ordering},
    vec::IntoIter,
};

use ritec_core::Ident;

#[derive(Clone, Debug)]
pub struct Generic {
    pub ident: Ident,
    id: usize,
}

impl Generic {
    #[inline]
    pub fn new(ident: Ident) -> Self {
        static ID: AtomicUsize = AtomicUsize::new(0);

        Self {
            ident,
            id: ID.fetch_add(1, Ordering::SeqCst),
        }
    }
}

impl PartialEq for Generic {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Generic {}

impl Hash for Generic {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Deref for Generic {
    type Target = Ident;

    fn deref(&self) -> &Self::Target {
        &self.ident
    }
}

impl Display for Generic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ident)
    }
}

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
