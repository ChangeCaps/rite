use crate::{Expr, LocalId};

#[derive(Clone, Debug, PartialEq)]
pub struct LetStmt {
    pub local: LocalId,
    pub value: Option<Expr>,
}
