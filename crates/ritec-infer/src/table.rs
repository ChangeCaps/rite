use std::collections::HashMap;

use ritec_core::trace;
use ritec_error::Diagnostic;
use ritec_hir as hir;

use crate::{
    InferType, Modification, Modifications, TypeVariable, TypeVariableKind, Unifier, UnifyResult,
};

#[derive(Clone, Debug, PartialEq)]
pub struct InferenceTable {
    variables: HashMap<InferType, InferType>,
    identifed: HashMap<hir::HirId, InferType>,
    generics: HashMap<hir::HirId, Vec<InferType>>,
    classes: HashMap<hir::HirId, hir::ClassId>,
    fields: HashMap<hir::HirId, hir::FieldId>,
    methods: HashMap<hir::HirId, hir::MethodId>,
    modifications: HashMap<hir::HirId, Modifications>,
    next_variable: usize,
}

impl InferenceTable {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            identifed: HashMap::new(),
            generics: HashMap::new(),
            classes: HashMap::new(),
            fields: HashMap::new(),
            methods: HashMap::new(),
            modifications: HashMap::new(),
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

    pub fn register_generic(&mut self, id: hir::HirId, ty: InferType) {
        self.generics.entry(id).or_default().push(ty);
    }

    pub fn get_generics(&self, id: hir::HirId) -> &Vec<InferType> {
        const EMPTY: &'static Vec<InferType> = &Vec::new();
        self.generics.get(&id).unwrap_or(EMPTY)
    }

    pub fn register_class(&mut self, id: hir::HirId, class: hir::ClassId) {
        self.classes.insert(id, class);
    }

    pub fn get_class(&self, id: hir::HirId) -> Option<hir::ClassId> {
        self.classes.get(&id).copied()
    }

    pub fn register_field(&mut self, id: hir::HirId, field: hir::FieldId) {
        self.fields.insert(id, field);
    }

    pub fn get_field(&self, id: hir::HirId) -> Option<hir::FieldId> {
        self.fields.get(&id).copied()
    }

    pub fn register_method(&mut self, id: hir::HirId, method: hir::MethodId) {
        self.methods.insert(id, method);
    }

    pub fn get_method(&self, id: hir::HirId) -> Option<hir::MethodId> {
        self.methods.get(&id).copied()
    }

    pub fn push_modification(&mut self, id: hir::HirId, modification: Modification) {
        self.modifications.entry(id).or_default().push(modification);
    }

    pub fn get_modifications(&self, id: hir::HirId) -> Option<&Modifications> {
        self.modifications.get(&id)
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

    pub fn unify(&mut self, a: &InferType, b: &InferType) -> Result<UnifyResult, Diagnostic> {
        let mut unifier = Unifier::new(self);

        unifier.unify(a, b)?;

        Ok(unifier.finish())
    }
}
