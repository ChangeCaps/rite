use crate::{TypeApplication, TypeVariable};

#[derive(Clone, Debug, PartialEq)]
pub enum InferError {
    ArgumentCount(TypeApplication, TypeApplication),
    Mismatch(TypeApplication, TypeApplication),
    OccursCheck(TypeVariable, TypeApplication),
}
