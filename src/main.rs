use std::{fs, io, path::PathBuf, sync::Mutex};

use rite::{
    diagnostic::{DiagnosticWriter, Source, Sources},
    parse::{self, Parser},
};

fn main() {
    let reporter = Mutex::new(Vec::new());
    let mut sources = Sources::new();

    let source = sources.push(Source {
        path: PathBuf::from("test.rite"),
        contents: fs::read_to_string("test.rite").unwrap(),
    });

    let input = &sources[source].contents;
    let tokens = parse::lex(&reporter, source, input);
    let mut parser = Parser::new(&reporter, &tokens);
    let expr = parse::expr(&mut parser);

    let mut writer = DiagnosticWriter::new(&sources, io::stdout());
    for diagnostic in reporter.into_inner().into_iter().flatten() {
        writer.write(&diagnostic).unwrap();
    }
}
