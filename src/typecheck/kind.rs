use crate::{rename, typecheck::State, var::Var};

#[derive(Clone, Debug)]
pub enum Kind {
    Inter(Vec<Kind>),
    Arity(usize),
    Trait(Var),
}

impl Kind {
    pub fn inter(&mut self, other: Kind) {
        match self {
            Kind::Inter(kinds) => match other {
                Kind::Inter(other) => kinds.extend(other),
                other => kinds.push(other),
            },

            _ => {
                *self = Kind::Inter(vec![self.clone(), other]);
            }
        }
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
        Kind::Trait(kind.var)
    }

    fn arity_kind(&mut self, kind: rename::ArityKind) -> Kind {
        Kind::Arity(kind.arity)
    }

    fn error_kind(&mut self, _kind: rename::ErrorKind) -> Kind {
        todo!()
    }
}
