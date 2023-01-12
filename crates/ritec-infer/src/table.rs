use std::collections::HashMap;

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

    pub fn next_variable(&mut self) -> TypeVariable {
        let variable = TypeVariable::new(self.next_variable);
        self.next_variable += 1;

        variable
    }

    pub fn normalize(&mut self, ty: &InferType) -> Option<InferType> {
        let InferType::Var(var) = ty else {
            return None;
        };

        self.variables.get(var).cloned()
    }

    pub fn substite(&mut self, variable: TypeVariable, ty: InferType) {
        self.variables.insert(variable, ty);
    }

    pub fn unify(&mut self, a: &InferType, b: &InferType) -> Result<UnifyResult, InferError> {
        let mut unifier = Unifier::new(self);

        unifier.unify(a, b)?;

        Ok(unifier.finish())
    }
}
