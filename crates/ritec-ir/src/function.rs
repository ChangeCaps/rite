use ritec_arena::Id;
use ritec_span::Ident;

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

pub type FunctionId = Id<Function>;
