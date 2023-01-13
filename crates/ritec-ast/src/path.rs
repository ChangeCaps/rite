use std::hash::{Hash, Hasher};

use ritec_core::{Ident, Span};

use crate::Type;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ItemSegment {
    pub ident: Ident,
    pub generics: Vec<Type>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PathSegment {
    Item(ItemSegment),
    SuperSegment(Span),
    SelfSegment(Span),
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

        self.segments.last().map(|s| match s {
            PathSegment::Item(item) => &item.ident,
            PathSegment::SuperSegment(_) => todo!(),
            PathSegment::SelfSegment(_) => todo!(),
        })
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
