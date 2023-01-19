use crate::{InferType, TypeProjection};

#[derive(Clone, Debug, PartialEq)]
pub struct Unify {
    pub a: InferType,
    pub b: InferType,
}

impl Unify {
    pub fn new(a: impl Into<InferType>, b: impl Into<InferType>) -> Self {
        Self {
            a: a.into(),
            b: b.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Normalize {
    pub proj: TypeProjection,
    pub expected: InferType,
}

impl Normalize {
    pub fn new(proj: impl Into<TypeProjection>, expected: impl Into<InferType>) -> Self {
        Self {
            proj: proj.into(),
            expected: expected.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Constraint {
    Unify(Unify),
    Normalize(Normalize),
}

impl From<Unify> for Constraint {
    fn from(value: Unify) -> Self {
        Self::Unify(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Solution {
    pub is_solved: bool,
    pub constraint: Constraint,
}
