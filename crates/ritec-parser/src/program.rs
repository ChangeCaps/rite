use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use ritec_ast as ast;
use ritec_core::{Ident, SourceFile, SourceMap, Span};
use ritec_error::{Diagnostic, Emitter};

use crate::{ParseBuffer, TokenStream};

pub struct ProgramParser<'a> {
    pub source_map: &'a mut SourceMap,
    pub emitter: &'a mut dyn Emitter,
    pub modules: HashMap<PathBuf, ast::ModuleId>,
}

impl<'a> ProgramParser<'a> {
    pub fn new(source_map: &'a mut SourceMap, emitter: &'a mut dyn Emitter) -> Self {
        Self {
            source_map,
            emitter,
            modules: HashMap::new(),
        }
    }

    pub fn parse_program(&mut self, path: &Path) -> Result<ast::Program, ()> {
        // read the file
        let Ok(source) = fs::read_to_string(path) else {
            let err = Diagnostic::error("failed to read file")
                .with_msg("failed to read main file".to_string());

            self.emitter.emit(err);

            return Err(());
        };

        // register the file
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

        // create the program
        let file_name = path.file_stem().unwrap().to_str().unwrap();
        let module_name = Ident::new(file_name, Span::new(0, file_name.len(), file_id));
        let mut program = ast::Program::new(module_name);

        // register the root module
        self.modules.insert(path.to_path_buf(), program.root_module);

        // parse the root module
        let mut parser = ParseBuffer::new(&tokens, program.root_module);
        let items = match parser.parse::<ast::Items>() {
            Ok(items) => items,
            Err(err) => {
                self.emitter.emit(err);
                return Err(());
            }
        };

        // add the items to the root module
        for item in items.items {
            match item {
                ast::Item::Function(item) => {
                    let id = program.functions.push(item);
                    program.root_mut().functions.push(id);
                }
                ast::Item::Module(item) => {
                    let id = self.parse_module(&mut program, path, &item.ident)?;
                    program.root_mut().modules.push(id);
                }
            }
        }

        Ok(program)
    }

    pub fn parse_module(
        &mut self,
        program: &mut ast::Program,
        path: &Path,
        ident: &Ident,
    ) -> Result<ast::ModuleId, ()> {
        let path = path.with_file_name(ident.value()).with_extension("ri");

        // if the module has already been parsed, return the id
        if let Some(&id) = self.modules.get(&path) {
            return Ok(id);
        }

        // read the file
        let Ok(source) = fs::read_to_string(&path) else {
            let err = Diagnostic::error("failed to read file")
                .with_msg_span("failed to read module file".to_string(), ident.span());

            self.emitter.emit(err);

            return Err(());
        };

        // register the file
        let source_file = SourceFile {
            path: source.to_string(),
            text: source,
        };

        let file_id = self.source_map.insert(source_file);
        let source = self.source_map.get(file_id).unwrap();

        // lex the file
        let tokens = match TokenStream::lex(&source.text, Some(file_id)) {
            Ok(tokens) => tokens,
            Err(err) => {
                self.emitter.emit(err.into());
                return Err(());
            }
        };

        // add the module to the program
        let module = program.modules.push(ast::Module::new(ident.clone()));
        self.modules.insert(path.clone(), module);

        // parse the file
        let mut parser = ParseBuffer::new(&tokens, module);
        let items = match parser.parse::<ast::Items>() {
            Ok(items) => items,
            Err(err) => {
                self.emitter.emit(err);
                return Err(());
            }
        };

        // add the items to the module
        for item in items.items {
            match item {
                ast::Item::Function(item) => {
                    let id = program.functions.push(item);
                    program.modules[module].functions.push(id);
                }
                ast::Item::Module(item) => {
                    let id = self.parse_module(program, &path, &item.ident)?;
                    program.modules[module].modules.push(id);
                }
            }
        }

        Ok(module)
    }
}
