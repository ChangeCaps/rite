mod builder;
mod expr;
mod stmt;

pub use builder::*;
pub use expr::*;
pub use stmt::*;

use ritec_core::Arena;
use ritec_hir as hir;
use ritec_infer::Error as InferError;
use ritec_mir::Local;

#[derive(Clone, Debug, PartialEq)]
pub struct Thir {
    pub locals: Arena<Local>,
    pub exprs: Arena<Expr>,
    pub stmts: Arena<Stmt>,
}

impl Thir {
    pub const fn new() -> Self {
        Self {
            locals: Arena::new(),
            exprs: Arena::new(),
            stmts: Arena::new(),
        }
    }

    pub fn from_hir(hir: &hir::Body) -> Result<Self, InferError> {
        let mut builder = ThirBuilder::new(hir)?;
        builder.build()
    }
}
