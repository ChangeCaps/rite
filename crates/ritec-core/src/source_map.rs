use std::ops::Deref;

use crate::{Arena, Id};

pub type FileId = Id<SourceFile>;

#[derive(Clone, Debug)]
pub struct SourceFile {
    pub path: String,
    pub text: String,
}

#[derive(Clone, Debug, Default)]
pub struct SourceMap {
    arena: Arena<SourceFile>,
}

impl SourceMap {
    pub const fn new() -> Self {
        Self {
            arena: Arena::new(),
        }
    }

    pub fn contains_path(&self, path: &str) -> bool {
        self.arena.values().any(|file| file.path == path)
    }

    pub fn insert(&mut self, source_file: SourceFile) -> FileId {
        self.arena.push(source_file)
    }
}

impl Deref for SourceMap {
    type Target = Arena<SourceFile>;

    fn deref(&self) -> &Self::Target {
        &self.arena
    }
}
