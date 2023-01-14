use std::fmt::Display;

use ritec_core::{Id, Ident};

use crate::{Body, FunctionType, Generics, LocalId, Type};

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionArgument {
    /// The name of the argument.
    pub ident: Ident,
    /// The type of the argument.
    pub ty: Type,
    /// The local variable that represents the argument.
    pub local: LocalId,
}

impl Display for FunctionArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.ident, self.ty)
    }
}

pub type FunctionId = Id<Function>;

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    /// The name of the function.
    pub ident: Ident,
    /// The generics of the function.
    pub generics: Generics,
    /// The arguments of the function.
    pub arguments: Vec<FunctionArgument>,
    /// The return type of the function.
    pub return_type: Type,
    /// The body of the function.
    pub body: Body,
}

impl Function {
    pub fn ty(&self) -> FunctionType {
        let arguments: Vec<_> = self.arguments.iter().map(|arg| arg.ty.clone()).collect();
        FunctionType::new(arguments, self.return_type.clone())
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn {}{}(", self.ident, self.generics)?;

        for (i, arg) in self.arguments.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }

            write!(f, "{}", arg)?;
        }

        write!(f, ") -> {}", self.return_type)?;

        writeln!(f, " {{")?;
        writeln!(f, "{}", self.body)?;
        write!(f, "}}")
    }
}
