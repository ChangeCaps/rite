use crate::{
    rename,
    typecheck::{State, Ty},
    var::Var,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Kind {
    pub infer:  bool,
    pub subst:  Option<Ty>,
    pub arity:  Arity,
    pub traits: Vec<Var>,
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Field {
    pub name: String,
    pub ty:   Ty,
}

impl Default for Kind {
    fn default() -> Self {
        Self {
            infer:  true,
            subst:  None,
            arity:  Arity::min(0),
            traits: Vec::new(),
            fields: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Arity {
    pub arity: usize,
    pub exact: bool,
}

impl Arity {
    pub fn min(arity: usize) -> Self {
        Self {
            arity,
            exact: false,
        }
    }

    pub fn exact(arity: usize) -> Self {
        Self { arity, exact: true }
    }
}

impl Kind {
    pub fn arity(arity: Arity) -> Self {
        Self {
            arity,
            ..Default::default()
        }
    }

    pub fn field(name: String, ty: Ty) -> Self {
        Self {
            fields: vec![Field { name, ty }],
            ..Default::default()
        }
    }

    pub fn format_arity(arity: usize) -> String {
        (0..=arity).map(|_| "*").collect::<Vec<_>>().join(" -> ")
    }
}

impl State<'_> {
    pub fn kind(&mut self, kind: rename::Kind) -> Kind {
        match kind {
            rename::Kind::Var(kind) => self.var_kind(kind),
            rename::Kind::Arity(kind) => self.arity_kind(kind),
            rename::Kind::Error(kind) => self.error_kind(kind),
        }
    }

    fn var_kind(&mut self, kind: rename::VarKind) -> Kind {
        Kind {
            traits: vec![kind.var],
            ..Default::default()
        }
    }

    fn arity_kind(&mut self, kind: rename::ArityKind) -> Kind {
        Kind::arity(Arity::exact(kind.arity))
    }

    fn error_kind(&mut self, _kind: rename::ErrorKind) -> Kind {
        Kind::default()
    }
}
