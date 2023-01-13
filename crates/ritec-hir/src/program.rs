use ritec_core::Arena;

use crate::Function;

#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub functions: Arena<Function>,
}
