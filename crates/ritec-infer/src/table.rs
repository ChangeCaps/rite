use std::collections::HashMap;

use crate::{InferError, Type, TypeVariable, Unifier, UnifyResult};

#[derive(Clone, Debug, PartialEq)]
pub struct InferenceTable {
    variables: HashMap<TypeVariable, Type>,
}

impl InferenceTable {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn normalize(&mut self, ty: &Type) -> Option<Type> {
        let Type::Var(var) = ty else {
            return None;
        };

        self.variables.get(var).cloned()
    }

    pub fn substite(&mut self, variable: TypeVariable, ty: Type) {
        self.variables.insert(variable, ty);
    }

    pub fn unify(&mut self, a: &Type, b: &Type) -> Result<UnifyResult, InferError> {
        let mut unifier = Unifier::new(self);

        todo!()
    }
}
