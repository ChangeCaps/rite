mod expr;
mod item;

pub use expr::*;
pub use item::*;

pub trait Phase {
    type Name;
    type Type;
}

pub struct Parse;

impl Phase for Parse {
    type Name = String;
    type Type = ();
}

pub struct Rename;

impl Phase for Rename {
    type Name = ItemId;
    type Type = ();
}

pub struct Typecheck;
