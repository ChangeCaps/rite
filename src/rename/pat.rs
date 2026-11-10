use crate::{
    ast,
    diagnostic::{Diagnostic, Span},
    rename::{Scope, State, Value, ValueKind, Vis},
    var::Var,
};

pub enum Pat {
    Bind(BindPat),
    Cons(ConsPat),
    Error(ErrorPat),
}

pub struct BindPat {
    pub var:  Option<Var>,
    pub span: Span,
}

pub struct ConsPat {
    pub var:  Var,
    pub args: Vec<Pat>,
    pub span: Span,
}

pub struct ErrorPat {
    pub span: Span,
}

impl State<'_> {
    pub fn pat(&mut self, scope: &mut Scope, pat: ast::Pat) -> Pat {
        match pat {
            ast::Pat::Cons(pat) => self.cons_pat(scope, pat),
            ast::Pat::Error(pat) => self.error_pat(scope, pat),
        }
    }

    fn cons_pat(&mut self, scope: &mut Scope, pat: ast::ConsPat) -> Pat {
        let args = pat
            .args
            .into_iter()
            .map(|arg| self.pat(scope, arg))
            .collect::<Vec<_>>();

        if let Some(var) = self.find_cons(scope, pat.var.as_ref()).cloned() {
            return Pat::Cons(ConsPat {
                var,
                args,
                span: pat.span,
            });
        }

        if !args.is_empty() {
            let diagnostic = Diagnostic::error("undefined constructor").label(pat.span, "here");
            self.reporter.report(diagnostic);

            return Pat::Error(ErrorPat { span: pat.span });
        };

        let var = Var::from(pat.var);

        let value = Value {
            vis:  Vis::Local,
            var:  var.clone(),
            kind: ValueKind::Local,
        };

        scope.values.push(value);

        Pat::Bind(BindPat {
            var:  Some(var),
            span: pat.span,
        })
    }

    fn error_pat(&mut self, _scope: &mut Scope, pat: ast::ErrorPat) -> Pat {
        Pat::Error(ErrorPat { span: pat.span })
    }
}
