use std::sync::atomic::{AtomicU64, Ordering};

use crate::diagnostic::Span;

#[derive(Clone, Debug, PartialEq)]
pub struct Var {
    pub ident:  String,
    pub unique: Unique,
    pub span:   Span,
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
