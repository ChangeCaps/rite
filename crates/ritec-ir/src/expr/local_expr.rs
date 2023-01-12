use std::fmt::{self, Display};

use crate::LocalId;

#[derive(Clone, Debug, PartialEq)]
pub struct LocalExpr {
    pub local: LocalId,
}

impl Display for LocalExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "local[{}]", self.local.as_raw_index())
    }
}
