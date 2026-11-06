mod expr;
mod item;
mod stmt;
mod ty;

pub use expr::*;
pub use item::*;
pub use stmt::*;
pub use ty::*;

use crate::diagnostic::Span;

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

pub struct RenamedModule {
    pub assignments: Vec<Assignment<Rename>>,
}

pub struct Assignment<P>
where
    P: Phase,
{
    pub ty:   Option<TypeLit<P>>,
    pub var:  P::Var,
    pub expr: Expr<P>,
    pub span: Span,
}
