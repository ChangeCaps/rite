use crate::{InferError, InferenceTable, Type, TypeApplication, TypeVariable};

pub struct UnifyResult {}

pub struct Unifier<'a> {
    table: &'a mut InferenceTable,
}

impl<'a> Unifier<'a> {
    pub fn new(table: &'a mut InferenceTable) -> Self {
        Self { table }
    }

    pub fn unify(&mut self, a: &Type, b: &Type) -> Result<(), InferError> {
        if let Some(ty) = self.table.normalize(a) {
            return self.unify(&ty, b);
        } else if let Some(ty) = self.table.normalize(b) {
            return self.unify(a, &ty);
        }

        match (a, b) {
            (Type::Var(a), Type::Var(b)) => self.unify_var_var(a, b),
            (Type::Var(a), Type::Apply(b)) | (Type::Apply(b), Type::Var(a)) => {
                self.unify_var_apply(a, b)
            }
            (Type::Apply(_), Type::Apply(_)) => todo!(),
        }
    }

    pub fn unify_var_var(&mut self, a: &TypeVariable, b: &TypeVariable) -> Result<(), InferError> {
        if a == b {
            return Ok(());
        }

        self.table.substite(*a, Type::Var(*b));

        Ok(())
    }

    pub fn unify_var_apply(
        &mut self,
        a: &TypeVariable,
        b: &TypeApplication,
    ) -> Result<(), InferError> {
        if b.arguments.iter().any(|arg| arg == &Type::Var(*a)) {
            return Err(InferError::OccursCheck(a.clone(), b.clone()));
        }

        self.table.substite(*a, Type::Apply(b.clone()));

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
