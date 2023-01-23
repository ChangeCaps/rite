use std::io::{self, prelude::*};

use ritec_core::SourceMap;

use crate::Diagnostic;

pub struct ErrorWriter<'a> {
    pub source_map: &'a SourceMap,
}

impl<'a> ErrorWriter<'a> {
    pub fn new(source_map: &'a SourceMap) -> Self {
        Self { source_map }
    }

    pub fn write(&self, writer: &mut dyn Write, error: &Diagnostic) -> io::Result<()> {
        Ok(())
    }
}
