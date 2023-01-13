use std::{fs, path::PathBuf};

use clap::Parser;
use ritec_ast_lower::ProgramLowerer as AstLowerer;
use ritec_core::{SourceFile, SourceMap};
use ritec_hir as hir;
use ritec_hir_lower::ProgramLowerer as HirLowerer;
use ritec_mir as mir;
use ritec_parser::{ParseBuffer, TokenStream};

#[derive(Parser)]
pub struct Args {
    #[clap(default_value = "main.ri")]
    pub path: PathBuf,
}

fn main() {
    let args = Args::parse();

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
    program_lowerer.lower(&items).unwrap();

    let mut mir_program = mir::Program::new();
    let mut program_lowerer = HirLowerer::new(&mut mir_program, &mut emitter);
    program_lowerer.lower(&hir_program).unwrap();

    println!("{:#?}", mir_program);
}
