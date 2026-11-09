use std::{fs, io, path::PathBuf, sync::Mutex};

use rite::{
    diagnostic::{DiagnosticWriter, Source, Sources},
    lower,
    parse::{self, Parser},
    rename,
    runtime::Runtime,
    typecheck,
};

fn main() {
    let reporter = Mutex::new(Vec::new());
    let mut sources = Sources::new();

    let source = sources.push(Source {
        path:     PathBuf::from("test.rite"),
        contents: fs::read_to_string("test.rite").unwrap(),
    });

    let input = &sources[source].contents;
    let tokens = parse::lex(&reporter, source, input);
    let mut parser = Parser::new(&reporter, &tokens);
    let stmts = parse::stmts(&mut parser);

    let module = rename::rename(&reporter, stmts);
    let module = typecheck::typecheck(&reporter, module);

    let mut writer = DiagnosticWriter::new(&sources, io::stdout());
    for diagnostic in reporter.lock().as_deref().into_iter().flatten() {
        writer.write(diagnostic).unwrap();
    }

    let expr = lower::lower(module);

    println!("{expr:?}");

    let mut runtime = Runtime::new();
    let output = runtime.run(&expr);

    println!("{output:?}");
}
