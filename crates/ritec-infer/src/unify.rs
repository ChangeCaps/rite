use ritec_core::trace;
use ritec_error::Diagnostic;

use crate::{
    Constraint, InferType, InferenceTable, Normalize, TypeApplication, TypeProjection, TypeVariable,
};

#[derive(Clone, Debug, PartialEq)]
pub struct UnifyResult {
    pub constraints: Vec<Constraint>,
}

pub struct Unifier<'a> {
    table: &'a mut InferenceTable,
    constraints: Vec<Constraint>,
}

impl<'a> Unifier<'a> {
    pub fn new(table: &'a mut InferenceTable) -> Self {
        Self {
            table,
            constraints: Vec::new(),
        }
    }

    pub fn finish(self) -> UnifyResult {
        UnifyResult {
            constraints: self.constraints,
        }
    }

    pub fn unify(&mut self, a: &InferType, b: &InferType) -> Result<(), Diagnostic> {
        if let Some(ty) = self.table.normalize_shallow(a) {
            return self.unify(&ty, b);
        } else if let Some(ty) = self.table.normalize_shallow(b) {
            return self.unify(a, &ty);
        }

        trace!("unify: {:?} = {:?}", a, b);

        match (a, b) {
            (InferType::Proj(a), InferType::Proj(b)) => self.unify_proj_proj(a, b),
            (InferType::Proj(a), b) | (b, InferType::Proj(a)) => self.unify_proj_ty(a, b),

            (InferType::Apply(a), InferType::Apply(b)) => self.unify_apply_apply(a, b),
            (InferType::Apply(a), InferType::Var(b)) | (InferType::Var(b), InferType::Apply(a)) => {
                self.unify_apply_var(a, b)
            }

            (InferType::Var(a), InferType::Var(b)) => self.unify_var_var(a, b),
            (InferType::Var(a), b) | (b, InferType::Var(a)) => self.unify_var_ty(a, b),
        }
    }

    pub fn unify_var_var(&mut self, a: &TypeVariable, b: &TypeVariable) -> Result<(), Diagnostic> {
        if a == b {
            return Ok(());
        }

        if !a.can_unify_with_var(&b) {
            let err = Diagnostic::error("cannot unify types");
            return Err(err);
        }

        (self.table).substitute(InferType::Var(*a), InferType::Var(*b));

        Ok(())
    }

    pub fn unify_proj_proj(
        &mut self,
        a: &TypeProjection,
        b: &TypeProjection,
    ) -> Result<(), Diagnostic> {
        let var = InferType::Var(self.table.new_variable(None));
        self.unify_proj_ty(a, &var)?;
        self.unify_proj_ty(b, &var)?;

        Ok(())
    }

    pub fn unify_proj_ty(&mut self, a: &TypeProjection, b: &InferType) -> Result<(), Diagnostic> {
        let noramlize = Normalize {
            proj: a.clone(),
            expected: b.clone(),
        };

        self.constraints.push(Constraint::Normalize(noramlize));

        Ok(())
    }

    pub fn unify_apply_apply(
        &mut self,
        a: &TypeApplication,
        b: &TypeApplication,
    ) -> Result<(), Diagnostic> {
        if a.item != b.item {
            let err = Diagnostic::error("cannot unify types");
            return Err(err);
        }

        if a.arguments.len() != b.arguments.len() {
            let err = Diagnostic::error("wrong number of arguments");
            return Err(err);
        }

        for (a, b) in a.arguments.iter().zip(b.arguments.iter()) {
            self.unify(a, b)?;
        }

        Ok(())
    }

    pub fn unify_apply_var(
        &mut self,
        a: &TypeApplication,
        b: &TypeVariable,
    ) -> Result<(), Diagnostic> {
        let mut arguments = Vec::new();

        for _ in 0..a.arguments.len() {
            let var = InferType::Var(self.table.new_variable(None));
            arguments.push(var);
        }

        let apply = TypeApplication {
            item: a.item.clone(),
            arguments,
            span: a.span,
        };
        self.unify_apply_apply(a, &apply)?;

        self.unify_var_ty(b, &InferType::Apply(apply))?;

        Ok(())
    }

    pub fn unify_var_ty(&mut self, a: &TypeVariable, b: &InferType) -> Result<(), Diagnostic> {
        if !a.can_unify_with(&b) {
            let err = Diagnostic::error("cannot unify types");
            return Err(err);
        }

        self.table.substitute(InferType::Var(a.clone()), b.clone());

        Ok(())
    }
}
