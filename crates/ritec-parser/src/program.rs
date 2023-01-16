use std::{fs, mem};

use ritec_ast as ast;
use ritec_core::{SourceFile, SourceMap};
use ritec_error::{Diagnostic, Emitter};

use crate::{ParseBuffer, TokenStream};

pub struct ProgramParser<'a> {
    pub emitter: &'a mut dyn Emitter,
    pub source_map: &'a mut SourceMap,
    pub program: ast::Program,
}

impl<'a> ProgramParser<'a> {
    pub fn new(emitter: &'a mut dyn Emitter, source_map: &'a mut SourceMap) -> Self {
        Self {
            emitter,
            source_map,
            program: ast::Program::new(),
        }
    }

    pub fn parse_program(&mut self, path: &str) -> Result<ast::Program, ()> {
        let Ok(source) = fs::read_to_string(path) else {
            let err = Diagnostic::error("failed to read file")
                .with_msg("failed to read main file".to_string());

            self.emitter.emit(err);

            return Err(());
        };

        let source_file = SourceFile {
            path: source.to_string(),
            text: source,
        };
        let file_id = self.source_map.insert(source_file);

        let source = self.source_map.get(file_id).unwrap();
        let tokens = match TokenStream::lex(&source.text, Some(file_id)) {
            Ok(tokens) => tokens,
            Err(err) => {
                self.emitter.emit(err.into());
                return Err(());
            }
        };

        let mut parser = ParseBuffer::new(&tokens);

        let items = match parser.parse::<ast::Items>() {
            Ok(items) => items,
            Err(err) => {
                self.emitter.emit(err);
                return Err(());
            }
        };

        Ok(mem::take(&mut self.program))
    }
}
