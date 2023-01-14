use core::fmt;
use std::{
    fmt::Display,
    hash::{Hash, Hasher},
    ops::Deref,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{Ident, Span};

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

    pub const fn span(&self) -> Span {
        self.ident.span()
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
