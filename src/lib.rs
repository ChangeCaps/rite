use std::{fs, path::Path};

use ritec_ast_lower::ProgramLowerer as AstLowerer;
use ritec_codegen_llvm::LLVMCodegen;
use ritec_core::{SourceFile, SourceMap};
use ritec_hir as hir;
use ritec_mir_build::ProgramBuilder;
use ritec_parser::{ParseBuffer, TokenStream};

pub struct Compiler {}

impl Compiler {
    pub fn compile(path: impl AsRef<Path>) {
        let source = fs::read_to_string(&path).unwrap();
        let source_file = SourceFile {
            path: path.as_ref().display().to_string(),
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

        let program_builder = ProgramBuilder::new(&hir_program);
        let res = program_builder.build();

        for diagnostic in emitter.iter() {
            println!("{:?}", diagnostic);
        }

        let mir = res.unwrap();

        LLVMCodegen::compile(&mir);
    }
}
