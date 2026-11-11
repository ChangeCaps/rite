use std::{ffi, fs, io, path::Path, sync::Mutex};

use crate::{
    ast,
    diagnostic::{Diagnostic, DiagnosticWriter, Source, Sources},
    ir, lower, parse, rename, typecheck,
};

#[derive(Default)]
pub struct Compiler {
    reporter: Mutex<Vec<Diagnostic>>,
    sources:  Sources,
    modules:  Vec<ast::Module>,
}

#[derive(Debug)]
pub struct CompileError;

impl Compiler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_module(&mut self, name: impl AsRef<str>, path: impl AsRef<Path>) -> io::Result<()> {
        let name = name.as_ref();
        let path = path.as_ref();

        let name = format!("{name}:");

        if path.is_file() {
            self.add_file(&name, path)
        } else if path.is_dir() {
            self.add_dir(&name, path)
        } else {
            Ok(())
        }
    }

    pub fn lower(self) -> Result<ir::Expr, CompileError> {
        let unit = rename::rename(&self.reporter, self.modules);
        let unit = typecheck::typecheck(&self.reporter, unit);

        let mut writer = DiagnosticWriter::new(&self.sources, io::stderr());
        let diagnostics = self.reporter.lock().unwrap();

        let result = writer.write(&diagnostics);
        if result.is_err() || result.is_ok_and(|success| !success) {
            return Err(CompileError);
        }

        let expr = lower::lower(unit);

        Ok(expr)
    }

    fn add_path(&mut self, name: &str, path: &Path) -> io::Result<()> {
        if path.is_file() {
            return self.add_file(name, path);
        }

        if path.is_dir() {
            let Some(file_name) = path.file_name().and_then(ffi::OsStr::to_str) else {
                return Ok(());
            };

            let name = format!("{name}{file_name}/");
            return self.add_dir(&name, path);
        }

        Ok(())
    }

    fn add_dir(&mut self, name: &str, path: &Path) -> io::Result<()> {
        for entry in path.read_dir()? {
            self.add_path(name, &entry?.path())?;
        }

        Ok(())
    }

    fn add_file(&mut self, name: &str, path: &Path) -> io::Result<()> {
        debug_assert!(path.is_file());

        if path.extension() != Some("ri".as_ref()) {
            return Ok(());
        };

        let Some(file_stem) = path.file_stem().and_then(ffi::OsStr::to_str) else {
            return Ok(());
        };

        let name = format!("{name}{file_stem}");

        let source = self.sources.push(Source {
            path:     path.to_path_buf(),
            contents: fs::read_to_string(path)?,
        });

        let input = &self.sources[source].contents;

        let module = parse::module(&self.reporter, source, input, name);

        self.modules.push(module);

        Ok(())
    }
}
