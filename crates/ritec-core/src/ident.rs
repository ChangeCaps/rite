use std::{
    cmp::Ordering,
    fmt::{self, Debug, Display},
    hash::{Hash, Hasher},
    ops::Deref,
    sync::Arc,
};

use crate::Span;

#[derive(Clone)]
pub struct Ident {
    value: Arc<str>,
    span: Span,
}

impl Ident {
    pub fn new(name: impl Into<Arc<str>>, span: Span) -> Self {
        let ident = Self {
            value: name.into(),
            span,
        };

        debug_assert!(ident.is_valid());

        ident
    }

    pub fn blank() -> Self {
        Self::new("_", Span::DUMMY)
    }

    pub fn is_blank(&self) -> bool {
        self.value() == "_"
    }

    pub fn is_valid(&self) -> bool {
        self.value.chars().all(|c| c.is_alphanumeric() || c == '_')
    }

    pub const fn span(&self) -> Span {
        self.span
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl PartialEq for Ident {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Ident {}

impl PartialOrd for Ident {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl Ord for Ident {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl Hash for Ident {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl From<&str> for Ident {
    fn from(value: &str) -> Self {
        Self::new(value, Span::DUMMY)
    }
}

impl Deref for Ident {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl Debug for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} > {:?}", self.value, self.span())
    }
}

impl Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
