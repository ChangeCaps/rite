use crate::{
    ast,
    diagnostic::{Diagnostic, Span},
    rename::State,
    var::Var,
};

pub enum Kind {
    Var(VarKind),
    Arity(ArityKind),
    Error(ErrorKind),
}

pub struct VarKind {
    pub var:  Var,
    pub span: Span,
}

pub struct ArityKind {
    pub arity: usize,
    pub span:  Span,
}

pub struct ErrorKind {
    pub span: Span,
}

impl State<'_> {
    pub fn kind(&mut self, kind: ast::Kind) -> Kind {
        match kind {
            ast::Kind::Var(kind) => self.var_kind(kind),
            ast::Kind::Arity(kind) => self.arity_kind(kind),
            ast::Kind::Error(kind) => self.error_kind(kind),
        }
    }

    fn var_kind(&mut self, kind: ast::VarKind) -> Kind {
        match self.find_kind(kind.var.as_ref()) {
            Some(var) => Kind::Var(VarKind {
                var:  var.clone(),
                span: kind.span,
            }),

            None => {
                let diagnostic = Diagnostic::error("undefined kind").label(kind.span, "found here");
                self.reporter.report(diagnostic);

                Kind::Error(ErrorKind { span: kind.span })
            }
        }
    }

    fn arity_kind(&mut self, kind: ast::ArityKind) -> Kind {
        Kind::Arity(ArityKind {
            arity: kind.arity,
            span:  kind.span,
        })
    }

    fn error_kind(&mut self, kind: ast::ErrorKind) -> Kind {
        Kind::Error(ErrorKind { span: kind.span })
    }
}
