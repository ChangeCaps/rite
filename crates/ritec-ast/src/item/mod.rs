mod function;

pub use function::*;
use ritec_span::Span;

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
