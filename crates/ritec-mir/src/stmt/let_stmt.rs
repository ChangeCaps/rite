use crate::{ExprId, LocalId};

#[derive(Clone, Debug, PartialEq)]
pub struct LetStmt {
    pub local: LocalId,
    pub init: Option<ExprId>,
}
