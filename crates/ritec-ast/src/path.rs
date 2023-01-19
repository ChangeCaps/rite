use std::{
    fmt::{self, Display},
    hash::{Hash, Hasher},
};

use ritec_core::{Ident, Span};

use crate::Type;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ItemSegment {
    pub ident: Ident,
    pub generics: Vec<Type>,
}

impl Display for ItemSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ident)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PathSegment {
    Item(ItemSegment),
    SuperSegment(Span),
    SelfSegment(Span),
}

impl Display for PathSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Item(item) => write!(f, "{}", item),
            Self::SuperSegment(_) => write!(f, "super"),
            Self::SelfSegment(_) => write!(f, "self"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Path {
    pub is_absolute: bool,
    pub segments: Vec<PathSegment>,
    pub span: Span,
}

impl Path {
    pub const fn is_absolute(&self) -> bool {
        self.is_absolute
    }

    pub const fn is_relative(&self) -> bool {
        !self.is_absolute()
    }

    pub fn get_ident(&self) -> Option<&Ident> {
        if self.is_absolute() {
            return None;
        }

        if self.segments.len() != 1 {
            return None;
        }

        match self.segments.last() {
            Some(PathSegment::Item(item)) => Some(&item.ident),
            _ => None,
        }
    }

    pub fn is_self(&self) -> bool {
        if self.is_absolute() {
            return false;
        }

        if self.segments.len() != 1 {
            return false;
        }

        match self.segments.last() {
            Some(PathSegment::SelfSegment(_)) => true,
            Some(_) => false,
            None => false,
        }
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        self.is_absolute == other.is_absolute && self.segments == other.segments
    }
}

impl Eq for Path {}

impl Hash for Path {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.is_absolute.hash(state);
        self.segments.hash(state);
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_absolute() {
            write!(f, "::")?;
        }

        let segments: Vec<_> = self.segments.iter().map(|s| s.to_string()).collect();
        write!(f, "{}", segments.join("::"))
    }
}
