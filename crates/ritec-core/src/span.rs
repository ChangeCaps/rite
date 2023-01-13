use std::{
    fmt::{self, Debug},
    ops::{BitOr, BitOrAssign},
};

use crate::FileId;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    pub lo: usize,
    pub hi: usize,
    pub file: Option<FileId>,
}

impl Span {
    pub const DUMMY: Self = Self::dummy();

    pub const fn new(lo: usize, hi: usize, file_id: FileId) -> Self {
        Self {
            lo,
            hi,
            file: Some(file_id),
        }
    }

    pub const fn dummy() -> Self {
        Self {
            lo: 0,
            hi: 0,
            file: None,
        }
    }

    pub const fn lo(&self) -> usize {
        self.lo
    }

    pub const fn hi(&self) -> usize {
        self.hi
    }

    pub const fn file(&self) -> Option<FileId> {
        self.file
    }

    pub const fn is_dummy(&self) -> bool {
        self.file.is_none()
    }

    pub const fn shrink_to_lo(&self) -> Self {
        Self {
            lo: self.lo,
            hi: self.lo,
            file: self.file,
        }
    }

    pub const fn shrink_to_hi(&self) -> Self {
        Self {
            lo: self.hi,
            hi: self.hi,
            file: self.file,
        }
    }

    pub const fn expand_lo(&self, length: usize) -> Self {
        Self {
            lo: self.lo - length,
            hi: self.hi,
            file: self.file,
        }
    }

    pub const fn expand_hi(&self, length: usize) -> Self {
        Self {
            lo: self.lo,
            hi: self.hi + length,
            file: self.file,
        }
    }
}

impl BitOr for Span {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        if let (Some(a), Some(b)) = (self.file, rhs.file) {
            debug_assert_eq!(a, b, "Spans should never be joined across files!");
        } else {
            debug_assert!(
                self.file.is_none() && rhs.file.is_none(),
                "Spans should never be joined across files!"
            );
        }

        Self {
            lo: self.lo.min(rhs.lo),
            hi: self.hi.max(rhs.hi),
            file: self.file,
        }
    }
}

impl BitOrAssign for Span {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.lo, self.hi)?;

        if let Some(file) = self.file {
            write!(f, ": file[{}]", file.as_raw_index())?;
        }

        Ok(())
    }
}
