use ritec_core::{Generic, Ident, Span};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Generics {
    pub params: Vec<Generic>,
    pub span: Span,
}

impl Generics {
    pub fn new(params: impl Into<Vec<Generic>>, span: Span) -> Self {
        Self {
            params: params.into(),
            span,
        }
    }

    pub fn empty(span: Span) -> Self {
        Self::new(Vec::new(), span)
    }

    pub fn get_ident(&self, ident: &Ident) -> Option<&Generic> {
        self.params.iter().find(|g| g.ident == *ident)
    }
}
