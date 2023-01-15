use std::{collections::HashMap, ops::Deref};

use inkwell::{
    context::Context, execution_engine::ExecutionEngine, module::Module, targets::TargetData,
    values::FunctionValue, OptimizationLevel,
};
use ritec_mir as mir;

use crate::FunctionBuilder;

pub struct CodegenCx<'c> {
    pub context: &'c Context,
    pub module: Module<'c>,
    pub execution_engine: ExecutionEngine<'c>,
    pub program: &'c mir::Program,
    pub functions: HashMap<(mir::FunctionId, Vec<mir::Type>), FunctionValue<'c>>,
}

impl<'c> CodegenCx<'c> {
    pub fn new(context: &'c Context, program: &'c mir::Program) -> Self {
        let module = context.create_module("main");
        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();

        Self {
            context,
            module,
            execution_engine,
            program,
            functions: HashMap::new(),
        }
    }

    pub fn target_data(&self) -> &TargetData {
        self.execution_engine.get_target_data()
    }

    pub fn build_function(
        &mut self,
        function: mir::FunctionId,
        generics: &[mir::Type],
    ) -> FunctionValue<'c> {
        let instance = (function, generics.to_vec());

        if let Some(function) = self.functions.get(&instance) {
            return *function;
        }

        let function = &self.program[function];
        let mut builder = FunctionBuilder::new(self, function, generics);
        let value = builder.build();

        self.functions.insert(instance, value);
        value
    }
}

impl<'a> Deref for CodegenCx<'a> {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        self.context
    }
}
