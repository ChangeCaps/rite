use std::{
    fmt,
    hash::{Hash, Hasher},
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use crate::diagnostic::Span;

#[derive(Clone)]
pub struct Ident {
    name: Arc<str>,
    span: Span,
}

impl Ident {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into().into(),
            span,
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn as_str(&self) -> &str {
        self.name.as_ref()
    }
}

impl fmt::Debug for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl PartialEq for Ident {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl PartialEq<str> for Ident {
    fn eq(&self, other: &str) -> bool {
        self.name.as_ref() == other
    }
}

impl Deref for Ident {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

#[derive(Clone)]
pub struct Var {
    ident:  Option<Ident>,
    unique: Unique,
}

impl From<Option<Ident>> for Var {
    fn from(ident: Option<Ident>) -> Self {
        Self {
            ident,
            unique: Unique::new(),
        }
    }
}

impl Var {
    pub fn new(ident: Ident) -> Self {
        Self {
            ident:  Some(ident),
            unique: Unique::new(),
        }
    }

    pub fn unnamed() -> Self {
        Self {
            ident:  None,
            unique: Unique::new(),
        }
    }

    pub fn ident(&self) -> Option<&Ident> {
        self.ident.as_ref()
    }

    pub fn unique(&self) -> Unique {
        self.unique
    }

    pub fn span(&self) -> Option<Span> {
        self.ident().map(Ident::span)
    }

    pub fn as_str(&self) -> Option<&str> {
        self.ident().map(Ident::as_str)
    }
}

impl PartialEq for Var {
    fn eq(&self, other: &Self) -> bool {
        self.unique == other.unique
    }
}

impl Eq for Var {}

impl Hash for Var {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.unique.hash(state);
    }
}

impl fmt::Debug for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_str() {
            Some(name) => write!(f, "{}:{}", name, self.unique.data),
            None => write!(f, "?:{}", self.unique.data),
        }
    }
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_str() {
            Some(name) => f.write_str(name),
            None => f.write_str("?"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unique {
    data: u64,
}

impl Default for Unique {
    fn default() -> Self {
        Self::new()
    }
}

impl Unique {
    pub fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);

        Self {
            data: NEXT.fetch_add(1, Ordering::SeqCst),
        }
    }
}

impl fmt::Display for Unique {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}
