use std::{fs, path::PathBuf};

use clap::Parser;
use ritec_ir::Program;
use ritec_lower::ProgramLowerer;
use ritec_parser::{ParseBuffer, TokenStream};
use ritec_span::{SourceFile, SourceMap};

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
    let stmt: ritec_ast::Items = parser.parse().unwrap();

    let mut program = Program::new();
    let mut lowerer = ProgramLowerer::new(&mut program);
    lowerer.lower_items(&stmt).unwrap();

    println!("{:#?}", program);
}
