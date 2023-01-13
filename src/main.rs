use std::{fs, path::PathBuf};

use clap::Parser;
use ritec_core::{SourceFile, SourceMap};
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

    println!("{:#?}", items);
}
