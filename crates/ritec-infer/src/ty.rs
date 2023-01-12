use ritec_ir::IntType;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeVariable {
    pub depth: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ItemName {
    Int(IntType),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypeApplication {
    pub item: ItemName,
    pub arguments: Vec<Type>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Var(TypeVariable),
    Apply(TypeApplication),
}
