use ritec_core::{Id, Ident, Span};

use crate::{Body, FunctionType, LocalId, Type};

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionArgument {
    pub ident: Ident,
    pub ty: Type,
    pub local: LocalId,
    pub span: Span,
}

pub type FunctionId = Id<Function>;

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub ident: Ident,
    pub arguments: Vec<FunctionArgument>,
    pub return_type: Type,
    pub body: Body,
}

impl Function {
    pub fn ty(&self) -> FunctionType {
        let mut span = self.ident.span();
        let mut arguments = Vec::with_capacity(self.arguments.len());
        for argument in &self.arguments {
            span |= argument.span;
            arguments.push(argument.ty.clone());
        }

        if !self.return_type.span().is_dummy() {
            span |= self.return_type.span();
        }

        FunctionType {
            arguments,
            return_type: Box::new(self.return_type.clone()),
            span,
        }
    }
}
