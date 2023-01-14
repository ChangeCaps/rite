use core::fmt;
use std::fmt::Display;

use ritec_core::Id;

use crate::LocalId;

#[derive(Clone, Debug, PartialEq)]
pub enum Projection {
    Deref,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Place {
    pub local: LocalId,
    pub proj: Vec<Projection>,
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
pub enum Operand {
    Copy(Place),
    Move(Place),
    Void,
}

impl Operand {
    pub fn as_place(&self) -> Option<&Place> {
        match self {
            Operand::Copy(place) => Some(place),
            Operand::Move(place) => Some(place),
            Operand::Void => None,
        }
    }

    pub fn to_place(&self) -> Option<Place> {
        match self {
            Operand::Copy(place) => Some(place.clone()),
            Operand::Move(place) => Some(place.clone()),
            Operand::Void => None,
        }
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Copy(place) => write!(f, "copy {}", place),
            Self::Move(place) => write!(f, "{}", place),
            Self::Void => write!(f, "void"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Operand(Operand),
    Address(Place),
}

impl Value {
    pub const VOID: Self = Self::Operand(Operand::Void);

    pub fn as_operand(&self) -> Option<&Operand> {
        match self {
            Self::Operand(operand) => Some(operand),
            _ => None,
        }
    }

    pub fn to_operand(self) -> Option<Operand> {
        match self {
            Self::Operand(operand) => Some(operand),
            _ => None,
        }
    }

    pub fn as_place(&self) -> Option<&Place> {
        self.as_operand()?.as_place()
    }

    pub fn to_place(self) -> Option<Place> {
        self.to_operand()?.to_place()
    }

    pub fn move_operand(place: impl Into<Place>) -> Self {
        Self::Operand(Operand::Move(place.into()))
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Operand(operand) => write!(f, "{}", operand),
            Self::Address(place) => write!(f, "&{}", place),
        }
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

pub type StmtId = Id<Stmt>;

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Assign(Assign),
}

impl Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Assign(assign) => write!(f, "{}", assign),
        }
    }
}
