mod context;
mod function_builder;

use std::fs;

pub use context::*;
pub use function_builder::*;

use inkwell::{context::Context, targets::FileType};
use ritec_mir as mir;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct LLVMCodegen;

impl LLVMCodegen {
    pub fn compile(program: &mir::Program) {
        let context = Context::create();
        let cx = CodegenCx::new(&context, program);

        for function in program.functions.values() {
            cx.build_function(function);
        }

        cx.module.print_to_stderr();
        cx.module.verify().unwrap();

        let triple = cx.target_machine.get_triple();
        println!("triple: {}", triple);

        let buffer = cx
            .target_machine
            .write_to_memory_buffer(&cx.module, FileType::Object)
            .unwrap();

        fs::write("main.o", buffer.as_slice()).unwrap();
    }
}
