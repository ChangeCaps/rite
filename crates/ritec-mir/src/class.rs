use std::fmt::Display;

use ritec_core::{Generic, Id, Ident};

use crate::{FunctionId, Type};

#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub ident: Ident,
    pub ty: Type,
    pub init: Option<FunctionId>,
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.ident, self.ty)
    }
}

pub type ClassId = Id<Class>;

#[derive(Clone, Debug, PartialEq)]
pub struct Class {
    pub ident: Ident,
    pub generics: Vec<Generic>,
    pub fields: Vec<Field>,
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let generics: Vec<_> = self.generics.iter().map(Generic::to_string).collect();
        writeln!(f, "class {}<{}> {{", self.ident, generics.join(", "))?;

        for field in &self.fields {
            write!(f, "    {}", field)?;

            if let Some(init) = field.init {
                write!(f, " = func{}", init.as_raw_index())?;
            }

            writeln!(f, ",")?;
        }

        write!(f, "}}")
    }
}
