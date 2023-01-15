use crate::{InferType, TypeApplication, TypeVariable};

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    ArgumentCount(TypeApplication, TypeApplication),
    AmbiguousType(TypeVariable),
    Mismatch(InferType, InferType),
    OccursCheck(TypeVariable, TypeApplication),
}
