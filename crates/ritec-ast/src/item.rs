use ritec_core::{Id, Ident, Span};

use crate::{Block, Generics, ModuleId, Type};

#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    Module(ModuleItem),
    Class(Class),
    Function(Function),
}

impl Item {
    pub const fn span(&self) -> Span {
        match self {
            Item::Function(item) => item.span,
            Item::Class(item) => item.span,
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
pub struct Field {
    pub ident: Ident,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SelfArgument {
    Owned,
    Pointer,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Method {
    pub ident: Ident,
    pub generics: Generics,
    pub self_argument: Option<SelfArgument>,
    pub arguments: Vec<FunctionArgument>,
    pub return_type: Option<Type>,
    pub body: Block,
    pub span: Span,
}

pub type ClassId = Id<Class>;

#[derive(Clone, Debug, PartialEq)]
pub struct Class {
    pub module: ModuleId,
    pub ident: Ident,
    pub generics: Generics,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionArgument {
    pub ident: Ident,
    pub ty: Type,
    pub span: Span,
}

pub type FunctionId = Id<Function>;

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub module: ModuleId,
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
