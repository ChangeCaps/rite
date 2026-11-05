mod expr;
mod item;
mod stmt;
mod ty;

pub use expr::*;
pub use item::*;
pub use stmt::*;
pub use ty::*;

pub trait Phase {
    type Var;
    type Type;
}

pub struct Parse;

impl Phase for Parse {
    type Var = String;
    type Type = ();
}

pub struct Rename;

impl Phase for Rename {
    type Var = Var;
    type Type = ();
}

pub struct Typecheck;
