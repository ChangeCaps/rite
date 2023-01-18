use std::collections::HashMap;

use ritec_core::trace;
use ritec_hir as hir;
use ritec_mir as mir;

use crate::{Error, InferType, TypeVariable, TypeVariableKind, Unifier, UnifyResult};

#[derive(Clone, Debug, PartialEq)]
pub struct InferenceTable {
    variables: HashMap<InferType, InferType>,
    identifed: HashMap<hir::HirId, InferType>,
    generics: HashMap<(hir::HirId, usize), InferType>,
    fields: HashMap<hir::HirId, mir::FieldId>,
    next_variable: usize,
}

impl InferenceTable {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            identifed: HashMap::new(),
            generics: HashMap::new(),
            fields: HashMap::new(),
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

    pub fn register_generic(&mut self, id: hir::HirId, generic: usize, ty: InferType) {
        self.generics.insert((id, generic), ty);
    }

    pub fn get_generic(&self, id: hir::HirId, generic: usize) -> Option<&InferType> {
        self.generics.get(&(id, generic))
    }

    pub fn register_field(&mut self, id: hir::HirId, field: hir::FieldId) {
        self.fields.insert(id, field.cast());
    }

    pub fn get_field(&self, id: hir::HirId) -> Option<mir::FieldId> {
        self.fields.get(&id).copied()
    }

    pub fn normalize_shallow(&mut self, ty: &InferType) -> Option<InferType> {
        self.variables.get(ty).cloned()
    }

    pub fn substitute(&mut self, from: InferType, to: InferType) {
        trace!("substitute: {:?} -> {:?}", from, to);

        self.variables.insert(from, to);
    }

    pub fn get_substitution(&self, ty: &InferType) -> Option<InferType> {
        self.variables.get(ty).cloned()
    }

    pub fn unify(&mut self, a: &InferType, b: &InferType) -> Result<UnifyResult, Error> {
        let mut unifier = Unifier::new(self);

        unifier.unify(a, b)?;

        Ok(unifier.finish())
    }
}
