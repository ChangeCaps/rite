mod context;
mod function_builder;

pub use context::*;
pub use function_builder::*;

use inkwell::context::Context;
use ritec_mir as mir;

type MainFn = unsafe extern "C" fn(i32, *const *const i8) -> i32;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct LLVMCodegen;

impl LLVMCodegen {
    pub fn compile(program: &mir::Program) {
        let context = Context::create();
        let mut cx = CodegenCx::new(&context, program);

        for (id, function) in program.functions.iter() {
            if !function.generics.is_empty() {
                continue;
            }

            cx.build_function(id, &[]);
        }

        cx.module.print_to_stderr();
        cx.module.verify().unwrap();

        let main_address = cx.execution_engine.get_function_address("main").unwrap();

        let main = unsafe { std::mem::transmute::<_, MainFn>(main_address) };

        let args = std::env::args().collect::<Vec<_>>();
        let args = args.iter().map(|arg| arg.as_ptr()).collect::<Vec<_>>();

        let result = unsafe { main(args.len() as i32, args.as_ptr() as _) };
        println!("Result: {}", result);
    }
}
