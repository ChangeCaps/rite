use std::path::Path;

use ritec_ast_lower::ProgramLowerer as AstLowerer;
use ritec_codegen_llvm::LLVMCodegen;
use ritec_core::SourceMap;
use ritec_hir as hir;
use ritec_mir_build::ProgramBuilder;
use ritec_parser::ProgramParser;

pub struct Compiler {}

impl Compiler {
    pub fn compile(path: impl AsRef<Path>) {
        let mut emitter = Vec::new();

        let mut source_map = SourceMap::new();
        let mut program_parser = ProgramParser::new(&mut source_map, &mut emitter);
        let program = program_parser.parse_program(path.as_ref());

        for diagnostic in emitter.iter() {
            println!("{:?}", diagnostic);
        }

        let program = program.unwrap();

        let mut hir_program = hir::Program::new();
        hir_program.add_intrinsics();

        let mut program_lowerer = AstLowerer::new(&mut hir_program, &mut emitter);
        let res = program_lowerer.lower(&program);

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

        println!("MIR: {}", mir);

        LLVMCodegen::compile(&mir);
    }
}
