use crate::{TypeApplication, TypeVariable};

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    ArgumentCount(TypeApplication, TypeApplication),
    AmbiguousType(TypeVariable),
    Mismatch(TypeApplication, TypeApplication),
    OccursCheck(TypeVariable, TypeApplication),
}
