use std::{fs, path::PathBuf};

use clap::Parser;
use ritec_ast_lower::ProgramLowerer as AstLowerer;
use ritec_core::{SourceFile, SourceMap};
use ritec_hir as hir;
use ritec_mir_build::thir::Thir;
use ritec_parser::{ParseBuffer, TokenStream};
use tracing::Level;

#[derive(Parser)]
pub struct Args {
    #[clap(default_value = "main.ri")]
    pub path: PathBuf,
    #[clap(short, long, default_value = "info")]
    pub log_level: Level,
}

fn main() {
    let args = Args::parse();

    let layer = tracing_subscriber::fmt()
        .with_max_level(args.log_level)
        .finish();

    tracing::subscriber::set_global_default(layer).unwrap();

    let source = fs::read_to_string(&args.path).unwrap();
    let source_file = SourceFile {
        path: args.path.display().to_string(),
        source: source.clone(),
    };
    let mut source_map = SourceMap::new();
    let file = source_map.insert(source_file);

    let tokens = TokenStream::lex(&source, Some(file)).unwrap();
    let mut parser = ParseBuffer::new(&tokens);
    let items: ritec_ast::Items = parser.parse().unwrap();

    let mut hir_program = hir::Program::new();
    let mut emitter = Vec::new();
    let mut program_lowerer = AstLowerer::new(&mut hir_program, &mut emitter);
    let res = program_lowerer.lower(&items);

    for diagnostic in emitter.iter() {
        println!("{:?}", diagnostic);
    }

    res.unwrap();

    for function in hir_program.functions.values() {
        let thir = Thir::from_hir(&function.body).unwrap();

        println!("{:#?}", thir);
    }
}
