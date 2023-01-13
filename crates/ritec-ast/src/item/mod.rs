mod function;

pub use function::*;
use ritec_core::Span;

#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    Function(FunctionItem),
}

impl Item {
    pub const fn span(&self) -> Span {
        match self {
            Item::Function(item) => item.span,
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
