use std::collections::HashMap;

use crate::{
    ast,
    diagnostic::Reporter,
    var::{Ident, Unique, Var},
};

mod expr;
mod intrinsic;
mod kind;
mod pat;
mod stmt;
mod ty;

pub use expr::*;
pub use kind::*;
pub use pat::*;
pub use stmt::*;
pub use ty::*;

pub const PRELUDE: &[&str] = &[
    "std:flow",
    "std:math",
    "std:maybe",
    "std:monad",
    "std:result",
];

pub struct Unit {
    pub defs:   HashMap<Unique, Def>,
    pub types:  HashMap<Unique, Type>,
    pub traits: HashMap<Unique, Trait>,
    pub impls:  Vec<Impl>,
}

pub fn rename(reporter: &dyn Reporter, modules: Vec<ast::Module>) -> Unit {
    let mut state = State::new(reporter);
    state.modules(modules);

    Unit {
        defs:   state.defs,
        types:  state.types,
        traits: state.traits,
        impls:  state.impls,
    }
}

struct State<'a> {
    reporter: &'a dyn Reporter,
    modules:  HashMap<String, Scope>,
    defs:     HashMap<Unique, Def>,
    types:    HashMap<Unique, Type>,
    traits:   HashMap<Unique, Trait>,
    impls:    Vec<Impl>,
}

#[derive(Clone)]
struct Scope {
    imports: Vec<String>,
    values:  Vec<ScopeValue>,
    cons:    Vec<Var>,
    tys:     Vec<ScopeTy>,
    kinds:   Vec<Var>,
}

impl Scope {
    fn new() -> Self {
        Self {
            imports: Vec::new(),
            values:  Vec::new(),
            cons:    Vec::new(),
            tys:     Vec::new(),
            kinds:   Vec::new(),
        }
    }
}

#[derive(Clone)]
struct ScopeTy {
    name: Option<String>,
    kind: ScopeTyKind,
}

#[derive(Clone)]
enum ScopeTyKind {
    Var(Var),
    Prim(TyPrim),
}

#[derive(Clone)]
struct ScopeValue {
    vis:  Vis,
    var:  Var,
    kind: ScopeValueKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Vis {
    Local,
    Public,
}

#[derive(Clone, Debug)]
enum ScopeValueKind {
    Local,
    Def,
    Cons(Var),
    Method(Var),
    Import(String),
}

impl Scope {
    fn find_value(&self, name: Option<&str>) -> Option<&ScopeValue> {
        self.values
            .iter()
            .rev()
            .find(|value| value.var.ident().map(Ident::as_str) == name)
    }

    fn find_cons(&self, name: Option<&Ident>) -> Option<&Var> {
        self.cons.iter().rev().find(|var| var.ident() == name)
    }

    fn find_ty(&self, name: Option<&str>) -> Option<&ScopeTy> {
        self.tys
            .iter()
            .rev()
            .find(|var| var.name.as_deref() == name)
    }

    fn find_kind(&self, name: Option<&Ident>) -> Option<&Var> {
        self.kinds.iter().rev().find(|var| var.ident() == name)
    }
}

impl ScopeTy {
    fn ident(&self) -> Option<&Ident> {
        match self.kind {
            ScopeTyKind::Var(ref var) => var.ident(),
            ScopeTyKind::Prim(..) => None,
        }
    }
}

impl From<Var> for ScopeTy {
    fn from(var: Var) -> Self {
        Self {
            name: var.ident().map(|ident| ident.as_str().into()),
            kind: ScopeTyKind::Var(var),
        }
    }
}

impl<'a> State<'a> {
    fn new(reporter: &'a dyn Reporter) -> Self {
        Self {
            reporter,
            modules: HashMap::new(),
            defs: HashMap::new(),
            types: HashMap::new(),
            traits: HashMap::new(),
            impls: Vec::new(),
        }
    }

    fn find_value<'b>(&'b self, scope: &'b Scope, name: Option<&str>) -> Option<&'b ScopeValue> {
        scope.find_value(name).or_else(|| {
            scope.imports.iter().find_map(|import| {
                self.modules[import]
                    .find_value(name)
                    .filter(|value| value.vis == Vis::Public)
            })
        })
    }

    fn find_cons<'b>(&'b self, scope: &'b Scope, name: Option<&Ident>) -> Option<&'b Var> {
        scope.find_cons(name).or_else(|| {
            (scope.imports.iter()).find_map(|import| self.modules[import].find_cons(name))
        })
    }

    fn find_ty<'b>(&'b self, scope: &'b Scope, name: Option<&str>) -> Option<&'b ScopeTy> {
        scope.find_ty(name).or_else(|| {
            (scope.imports.iter()).find_map(|import| self.modules[import].find_ty(name))
        })
    }

    fn find_kind<'b>(&'b self, scope: &'b Scope, name: Option<&Ident>) -> Option<&'b Var> {
        scope.find_kind(name).or_else(|| {
            (scope.imports.iter()).find_map(|import| self.modules[import].find_kind(name))
        })
    }

    fn find_trait_member(&self, trait_: &Var, name: Option<&str>) -> Option<&TraitMethod> {
        self.traits
            .get(&trait_.unique())
            .unwrap()
            .methods
            .iter()
            .find(|member| member.var.ident().map(|ident| ident.as_str()) == name)
    }

    #[track_caller]
    fn type_mut(&mut self, var: &Var) -> &mut Type {
        self.types
            .get_mut(&var.unique())
            .expect("`type_mut` should be called with a valid `var`")
    }

    #[track_caller]
    fn trait_mut(&mut self, var: &Var) -> &mut Trait {
        self.traits
            .get_mut(&var.unique())
            .expect("`trait_mut` should be called with a valid `var`")
    }
}
