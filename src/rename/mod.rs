use std::collections::HashMap;

use crate::{
    ast,
    diagnostic::Reporter,
    var::{Ident, Unique, Var},
};

mod expr;
mod kind;
mod pat;
mod stmt;
mod ty;

pub use expr::*;
pub use kind::*;
pub use pat::*;
pub use stmt::*;
pub use ty::*;

pub struct Module {
    pub assignments: HashMap<Unique, Def>,
    pub types:       HashMap<Unique, Type>,
    pub traits:      HashMap<Unique, Trait>,
    pub impls:       Vec<Impl>,
}

pub fn rename(reporter: &dyn Reporter, stmts: Vec<ast::Stmt>) -> Module {
    let mut state = State::new(reporter);
    state.stmts(stmts);

    Module {
        assignments: state.defs,
        types:       state.types,
        traits:      state.traits,
        impls:       state.impls,
    }
}

struct State<'a> {
    reporter: &'a dyn Reporter,
    values:   Vec<(Var, ValueKind)>,
    cons:     Vec<Var>,
    tys:      Vec<Var>,
    kinds:    Vec<Var>,
    defs:     HashMap<Unique, Def>,
    types:    HashMap<Unique, Type>,
    traits:   HashMap<Unique, Trait>,
    impls:    Vec<Impl>,
}

#[derive(Clone, Debug)]
enum ValueKind {
    Local,
    Def,
    Cons(Var),
    Method(Var),
}

impl<'a> State<'a> {
    fn new(reporter: &'a dyn Reporter) -> Self {
        Self {
            reporter,
            values: Vec::new(),
            cons: Vec::new(),
            tys: Vec::new(),
            kinds: Vec::new(),
            defs: HashMap::new(),
            types: HashMap::new(),
            traits: HashMap::new(),
            impls: Vec::new(),
        }
    }

    fn value_scope<T>(&mut self, f: impl FnOnce(&mut State<'_>) -> T) -> T {
        let vars_len = self.values.len();

        let output = f(self);

        self.values.truncate(vars_len);

        output
    }

    fn ty_scope<T>(&mut self, f: impl FnOnce(&mut State<'_>) -> T) -> T {
        let tys_len = self.tys.len();

        let output = f(self);

        self.tys.truncate(tys_len);

        output
    }

    fn find_value(&self, name: Option<&Ident>) -> Option<&(Var, ValueKind)> {
        self.values
            .iter()
            .rev()
            .find(|(var, _)| var.ident() == name)
    }

    fn find_cons(&self, name: Option<&Ident>) -> Option<&Var> {
        self.cons.iter().rev().find(|var| var.ident() == name)
    }

    fn find_ty(&self, name: Option<&Ident>) -> Option<&Var> {
        self.tys.iter().rev().find(|var| var.ident() == name)
    }

    fn find_kind(&self, name: Option<&Ident>) -> Option<&Var> {
        self.kinds.iter().rev().find(|var| var.ident() == name)
    }

    fn find_trait_member(&self, trait_: &Var, name: Option<&str>) -> Option<&TraitMethod> {
        self.traits
            .get(&trait_.unique())
            .unwrap()
            .methods
            .iter()
            .find(|member| member.var.ident().map(|ident| ident.as_str()) == name)
    }
}
