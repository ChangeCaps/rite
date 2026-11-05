use std::{ops::Index, path::PathBuf};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SourceId {
    data: u32,
}

#[derive(Clone, Debug)]
pub struct Source {
    pub path:     PathBuf,
    pub contents: String,
}

#[derive(Clone, Debug)]
pub struct Sources {
    sources: Vec<Source>,
}

impl Default for Sources {
    fn default() -> Self {
        Self::new()
    }
}

impl Sources {
    pub const fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    pub fn push(&mut self, source: Source) -> SourceId {
        self.sources.push(source);
        let data = self.sources.len() - 1;
        SourceId { data: data as u32 }
    }
}

impl Index<SourceId> for Sources {
    type Output = Source;

    fn index(&self, index: SourceId) -> &Self::Output {
        &self.sources[index.data as usize]
    }
}
