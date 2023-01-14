use std::{collections::HashMap, ops::Deref};

use inkwell::{
    context::Context,
    module::Module,
    targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetData, TargetMachine},
    values::FunctionValue,
    OptimizationLevel,
};
use ritec_mir as mir;

use crate::FunctionBuilder;

pub struct CodegenCx<'a> {
    pub context: &'a Context,
    pub module: Module<'a>,
    pub target_machine: TargetMachine,
    pub program: &'a mir::Program,
    pub functions: HashMap<mir::FunctionId, FunctionValue<'a>>,
}

impl<'a> CodegenCx<'a> {
    pub fn new(context: &'a Context, program: &'a mir::Program) -> Self {
        let config = InitializationConfig::default();
        Target::initialize_x86(&config);
        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple).unwrap();

        let target_machine = target
            .create_target_machine(
                &target_triple,
                "x86-64",
                "+avx2",
                OptimizationLevel::None,
                RelocMode::Default,
                CodeModel::Default,
            )
            .unwrap();

        Self {
            context,
            program,
            module: context.create_module("main"),
            target_machine,
            functions: HashMap::new(),
        }
    }

    pub fn target_data(&self) -> TargetData {
        self.target_machine.get_target_data()
    }

    pub fn build_function(&self, function: &mir::Function) {
        let mut builder = FunctionBuilder::new(self, function);
        builder.build();
    }
}

impl<'a> Deref for CodegenCx<'a> {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        self.context
    }
}
