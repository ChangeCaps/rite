use std::collections::HashMap;

use ritec_core::trace;

use crate::{InferError, InferType, TypeVariable, Unifier, UnifyResult};

#[derive(Clone, Debug, PartialEq)]
pub struct InferenceTable {
    variables: HashMap<TypeVariable, InferType>,
    next_variable: usize,
}

impl InferenceTable {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            next_variable: 0,
        }
    }

    pub fn new_variable(&mut self) -> TypeVariable {
        let variable = TypeVariable {
            index: self.next_variable,
        };

        self.next_variable += 1;

        variable
    }

    pub fn normalize(&mut self, ty: &InferType) -> Option<InferType> {
        let InferType::Var(var) = ty else {
            return None;
        };

        self.variables.get(var).cloned()
    }

    pub fn substite(&mut self, var: TypeVariable, ty: InferType) {
        trace!("substite: {:?} -> {:?}", var, ty);

        self.variables.insert(var, ty);
    }

    pub fn get_substitution(&self, var: &TypeVariable) -> Option<InferType> {
        self.variables.get(var).cloned()
    }

    pub fn unify(&mut self, a: &InferType, b: &InferType) -> Result<UnifyResult, InferError> {
        let mut unifier = Unifier::new(self);

        unifier.unify(a, b)?;

        Ok(unifier.finish())
    }
}
