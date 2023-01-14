use ritec_hir as hir;
use ritec_infer::Solver;
use ritec_mir as mir;

use crate::{thir, Builder, Error};

pub struct ProgramBuilder<'a> {
    pub hir: &'a hir::Program,
    pub mir: mir::Program,
}

impl<'a> ProgramBuilder<'a> {
    pub fn new(program: &'a hir::Program) -> Self {
        Self {
            hir: program,
            mir: mir::Program::new(),
        }
    }

    pub fn build(mut self) -> Result<mir::Program, Error> {
        for function in self.hir.functions.values() {
            self.build_function(function)?;
        }

        Ok(self.mir)
    }

    pub fn build_function(&mut self, function: &hir::Function) -> Result<(), Error> {
        let mut solver = Solver::new();
        solver.set_return_type(function.return_type.clone());
        solver.solve_body(&function.body)?;

        let return_type = solver.resolve_return_type()?;

        let mut thir_builder = thir::ThirBuilder::new(&function.body, solver.finish())?;
        let thir = thir_builder.build()?;

        let builder = Builder::new(&thir);
        let mir = builder.build();

        let mut params = Vec::new();
        for param in &function.generics.params {
            params.push(param.clone());
        }

        let mut arguments = Vec::new();
        for argument in &function.arguments {
            let local = &thir[argument.local.cast::<mir::Local>()];

            let argument = mir::FunctionArgument {
                ident: argument.ident.clone(),
                ty: local.ty.clone(),
                local: argument.local.cast(),
            };

            arguments.push(argument);
        }

        let function = mir::Function {
            ident: function.ident.clone(),
            generics: mir::Generics::new(params),
            arguments,
            return_type,
            body: mir,
        };

        self.mir.functions.push(function);

        Ok(())
    }
}
