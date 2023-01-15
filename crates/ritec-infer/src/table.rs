use std::collections::HashMap;

use ritec_core::trace;
use ritec_hir as hir;

use crate::{Error, InferType, TypeVariable, TypeVariableKind, Unifier, UnifyResult};

#[derive(Clone, Debug, PartialEq)]
pub struct InferenceTable {
    variables: HashMap<TypeVariable, InferType>,
    identifed: HashMap<hir::HirId, InferType>,
    next_variable: usize,
}

impl InferenceTable {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            identifed: HashMap::new(),
            next_variable: 0,
        }
    }

    /// Creates a new [`TypeVariable`].
    pub fn new_variable(&mut self, kind: Option<TypeVariableKind>) -> TypeVariable {
        let variable = TypeVariable {
            index: self.next_variable,
            kind,
        };

        self.next_variable += 1;

        variable
    }

    /// Registers an [`InferType`] with a [`hir::HirId`].
    pub fn register_type(&mut self, id: hir::HirId, ty: InferType) {
        self.identifed.insert(id, ty);
    }

    pub fn get_type(&self, id: hir::HirId) -> Option<&InferType> {
        self.identifed.get(&id)
    }

    pub fn normalize_shallow(&mut self, ty: &InferType) -> Option<InferType> {
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

    pub fn unify(&mut self, a: &InferType, b: &InferType) -> Result<UnifyResult, Error> {
        let mut unifier = Unifier::new(self);

        unifier.unify(a, b)?;

        Ok(unifier.finish())
    }
}
