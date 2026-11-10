use std::{fs, io, path::PathBuf, sync::Mutex};

use rite::{
    diagnostic::{DiagnosticWriter, Source, Sources},
    lower, parse, rename,
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

    let module = parse::module(
        &reporter,
        source,
        input,
        String::from("test:test"),
    );

    let unit = rename::rename(&reporter, vec![module]);
    let unit = typecheck::typecheck(&reporter, unit);

    let mut writer = DiagnosticWriter::new(&sources, io::stdout());
    let diagnostics = reporter.lock().unwrap();
    writer.write(&diagnostics).unwrap();

    let expr = lower::lower(unit);

    println!("{expr:?}");

    let mut runtime = Runtime::new();
    let output = runtime.run(&expr);

    println!("{output:?}");
}
