use ritec_core::{Ident, Span};

use crate::{Block, Generics, Type};

#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    Function(Function),
    Module(ModuleItem),
}

impl Item {
    pub const fn span(&self) -> Span {
        match self {
            Item::Function(item) => item.span,
            Item::Module(item) => item.span,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Items {
    pub items: Vec<Item>,
    pub span: Span,
}

impl Items {
    pub fn new(items: Vec<Item>, span: Span) -> Self {
        Self { items, span }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Item> {
        self.items.iter()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionArgument {
    pub ident: Ident,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub ident: Ident,
    pub generics: Generics,
    pub arguments: Vec<FunctionArgument>,
    pub return_type: Option<Type>,
    pub body: Block,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModuleItem {
    pub ident: Ident,
    pub span: Span,
}
