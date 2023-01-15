use std::fmt::{self, Display};

use ritec_core::Id;

use crate::{LocalId, Type, Value};

#[derive(Clone, Debug, PartialEq)]
pub enum Projection {
    Deref,
}

impl Projection {
    pub fn apply_type(&self, ty: Type) -> Type {
        match self {
            Projection::Deref => match ty {
                Type::Pointer(ty) => *ty.pointee,
                _ => panic!("cannot deref non-pointer type"),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Place {
    pub local: LocalId,
    pub proj: Vec<Projection>,
}

impl Place {
    pub const fn local(local: LocalId) -> Self {
        Self {
            local,
            proj: Vec::new(),
        }
    }
}

impl From<LocalId> for Place {
    fn from(local: LocalId) -> Self {
        Self {
            local,
            proj: Vec::new(),
        }
    }
}

impl Display for Place {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = format!("_{}", self.local.as_raw_index());

        for proj in &self.proj {
            match proj {
                Projection::Deref => out = format!("*{}", out),
            }
        }

        write!(f, "{}", out)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Assign {
    pub place: Place,
    pub value: Value,
}

impl Display for Assign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.place, self.value)
    }
}

pub type StmtId = Id<Statement>;

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Assign(Assign),
    Drop(Value),
}

impl From<Assign> for Statement {
    fn from(assign: Assign) -> Self {
        Self::Assign(assign)
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Assign(assign) => write!(f, "{}", assign),
            Self::Drop(value) => write!(f, "drop {}", value),
        }
    }
}
