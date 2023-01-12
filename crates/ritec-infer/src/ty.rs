use ritec_ir::IntType;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeVariable {
    pub depth: usize,
}

impl TypeVariable {
    pub fn new(depth: usize) -> Self {
        Self { depth }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ItemId {
    Void,
    Bool,
    Int(IntType),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypeApplication {
    pub item: ItemId,
    pub arguments: Vec<InferType>,
}

impl TypeApplication {
    pub fn new(item: ItemId) -> Self {
        Self {
            item,
            arguments: Vec::new(),
        }
    }
}

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct TypeProjection {}

#[derive(Clone, Debug, PartialEq)]
pub enum InferType {
    Var(TypeVariable),
    Apply(TypeApplication),
    Proj(TypeProjection),
}

impl From<TypeVariable> for InferType {
    fn from(value: TypeVariable) -> Self {
        Self::Var(value)
    }
}

impl From<TypeApplication> for InferType {
    fn from(value: TypeApplication) -> Self {
        Self::Apply(value)
    }
}

impl From<TypeProjection> for InferType {
    fn from(value: TypeProjection) -> Self {
        Self::Proj(value)
    }
}
