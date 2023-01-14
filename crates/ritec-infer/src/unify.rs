use ritec_core::trace;

use crate::{
    Constraint, InferError, InferType, InferenceTable, Normalize, TypeApplication, TypeProjection,
    TypeVariable,
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

    pub fn unify(&mut self, a: &InferType, b: &InferType) -> Result<(), InferError> {
        if let Some(ty) = self.table.normalize(a) {
            return self.unify(&ty, b);
        } else if let Some(ty) = self.table.normalize(b) {
            return self.unify(a, &ty);
        }

        trace!("unify: {:?} = {:?}", a, b);

        match (a, b) {
            (InferType::Proj(a), InferType::Proj(b)) => self.unify_proj_proj(a, b),
            (InferType::Proj(a), b) | (b, InferType::Proj(a)) => self.unify_proj_ty(a, b),

            (InferType::Var(a), InferType::Var(b)) => self.unify_var_var(a, b),
            (InferType::Var(a), b) | (b, InferType::Var(a)) => self.unify_var_ty(a, b),

            (InferType::Apply(a), InferType::Apply(b)) => self.unify_apply_apply(a, b),
        }
    }

    pub fn unify_var_var(&mut self, a: &TypeVariable, b: &TypeVariable) -> Result<(), InferError> {
        if a == b {
            return Ok(());
        }

        self.table.substite(*a, InferType::Var(*b));

        Ok(())
    }

    pub fn unify_proj_proj(
        &mut self,
        a: &TypeProjection,
        b: &TypeProjection,
    ) -> Result<(), InferError> {
        let var = InferType::Var(self.table.new_variable());
        self.unify_proj_ty(a, &var)?;
        self.unify_proj_ty(b, &var)?;

        Ok(())
    }

    pub fn unify_proj_ty(&mut self, a: &TypeProjection, b: &InferType) -> Result<(), InferError> {
        let noramlize = Normalize {
            projection: a.clone(),
            expected: b.clone(),
        };

        self.constraints.push(Constraint::Normalize(noramlize));

        Ok(())
    }

    pub fn unify_var_ty(&mut self, a: &TypeVariable, b: &InferType) -> Result<(), InferError> {
        self.table.substite(a.clone(), b.clone());

        Ok(())
    }

    pub fn unify_apply_apply(
        &mut self,
        a: &TypeApplication,
        b: &TypeApplication,
    ) -> Result<(), InferError> {
        if a.item != b.item {
            return Err(InferError::Mismatch(a.clone(), b.clone()));
        }

        if a.arguments.len() != b.arguments.len() {
            return Err(InferError::ArgumentCount(a.clone(), b.clone()));
        }

        for (a, b) in a.arguments.iter().zip(b.arguments.iter()) {
            self.unify(a, b)?;
        }

        Ok(())
    }
}
