use std::fmt;

use crate::diagnostic::SourceId;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub source: SourceId,
    pub start:  u32,
    pub end:    u32,
}

impl Span {
    pub const DUMMY: Self = Self {
        source: SourceId::DUMMY,
        start:  u32::MAX,
        end:    u32::MAX,
    };

    pub const fn new(source: SourceId, start: u32, end: u32) -> Self {
        Self { source, start, end }
    }

    pub fn join(self, other: Self) -> Self {
        debug_assert_eq!(self.source, other.source);

        Self {
            source: self.source,
            start:  self.start.min(other.start),
            end:    self.end.max(other.end),
        }
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}
