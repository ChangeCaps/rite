use ritec_error::Diagnostic;
use ritec_hir as hir;
use ritec_infer::Solver;
use ritec_mir as mir;

use crate::{build_type, thir, FunctionBuilder};

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

    pub fn build(mut self) -> Result<mir::Program, Diagnostic> {
        for (id, class) in self.hir.classes.iter() {
            self.build_class(id, class)?;
        }

        for (id, function) in self.hir.functions.iter() {
            self.build_function(id, function)?;
        }

        Ok(self.mir)
    }

    pub fn build_class(&mut self, id: hir::ClassId, class: &hir::Class) -> Result<(), Diagnostic> {
        let mut generics = Vec::new();
        for generic in class.generics.params.iter() {
            generics.push(generic.clone());
        }

        let mut fields = Vec::new();
        for field in class.fields.values() {
            let ty = build_type(&field.ty);

            let field = mir::Field {
                ident: field.ident.clone(),
                ty,
                init: None,
            };

            fields.push(field);
        }

        let class = mir::Class {
            ident: class.ident.clone(),
            generics,
            fields,
        };

        self.mir.classes.insert(id.cast(), class);

        Ok(())
    }

    pub fn build_function(
        &mut self,
        id: hir::FunctionId,
        function: &hir::Function,
    ) -> Result<(), Diagnostic> {
        let mut solver = Solver::new(self.hir);
        solver.set_return_type(function.return_type.clone());
        solver.solve_body(&function.body)?;

        let return_type = solver.resolve_return_type()?;

        let mut thir_builder = thir::ThirBuilder::new(&self.hir, &function.body, solver.finish()?)?;
        let thir = thir_builder.build()?;

        let function_builder = FunctionBuilder::new(&thir, &self.hir.classes);
        let mir = function_builder.build();

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
            generics: params,
            arguments,
            return_type,
            body: mir,
        };

        self.mir.functions.insert(id.cast(), function);

        Ok(())
    }
}
