use crate::Type;

#[derive(Clone, Debug, PartialEq)]
pub struct Unify {
    pub a: Type,
    pub b: Type,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Constraint {
    Unify(Unify),
}
