use crate::{
    ast,
    diagnostic::{Diagnostic, Span},
    rename::{Scope, ScopeValue, ScopeValueKind, State, Vis},
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
    pub var:   Var,
    pub type_: Var,
    pub args:  Vec<Pat>,
    pub span:  Span,
}

pub struct ErrorPat {
    pub span: Span,
}

impl Pat {
    pub fn span(&self) -> Span {
        match self {
            Pat::Bind(pat) => pat.span,
            Pat::Cons(pat) => pat.span,
            Pat::Error(pat) => pat.span,
        }
    }
}

impl State<'_> {
    pub fn pat(&self, scope: &mut Scope, pat: ast::Pat) -> Pat {
        match pat {
            ast::Pat::Bin(pat) => self.bin_pat(scope, pat),
            ast::Pat::Wild(pat) => self.wild_pat(scope, pat),
            ast::Pat::Cons(pat) => self.cons_pat(scope, pat),
            ast::Pat::List(pat) => self.list_pat(scope, pat),
            ast::Pat::Paren(pat) => self.paren_pat(scope, pat),
            ast::Pat::Error(pat) => self.error_pat(scope, pat),
        }
    }

    fn bin_pat(&self, scope: &mut Scope, pat: ast::BinPat) -> Pat {
        let Some(found) = self.find_cons(scope, Some(pat.op.as_str())).cloned() else {
            let diagnostic = Diagnostic::error("operator not defined").label(
                pat.span,
                format!("`{}` used here", pat.op.as_str()),
            );

            self.reporter.report(diagnostic);

            return Pat::Error(ErrorPat { span: pat.span });
        };

        let lhs = self.pat(scope, *pat.lhs);
        let rhs = self.pat(scope, *pat.rhs);

        Pat::Cons(ConsPat {
            var:   found.var,
            type_: found.type_,
            args:  vec![lhs, rhs],
            span:  pat.span,
        })
    }

    fn wild_pat(&self, _scope: &mut Scope, pat: ast::WildPat) -> Pat {
        Pat::Bind(BindPat {
            var:  None,
            span: pat.span,
        })
    }

    fn cons_pat(&self, scope: &mut Scope, pat: ast::ConsPat) -> Pat {
        let args = pat
            .args
            .into_iter()
            .map(|arg| self.pat(scope, arg))
            .collect::<Vec<_>>();

        if let Some(cons) = self.find_cons(scope, pat.var.as_deref()).cloned() {
            return Pat::Cons(ConsPat {
                var: cons.var,
                type_: cons.type_,
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

        let value = ScopeValue {
            vis:  Vis::Local,
            var:  var.clone(),
            kind: ScopeValueKind::Local,
        };

        scope.values.push(value);

        Pat::Bind(BindPat {
            var:  Some(var),
            span: pat.span,
        })
    }

    fn list_pat(&self, scope: &mut Scope, pat: ast::ListPat) -> Pat {
        let Some(found) = self.find_cons(scope, Some("[]")).cloned() else {
            let diagnostic =
                Diagnostic::error("operator not defined").label(pat.span, "`[]` used here");

            self.reporter.report(diagnostic);

            return Pat::Error(ErrorPat { span: pat.span });
        };

        Pat::Cons(ConsPat {
            var:   found.var,
            type_: found.type_,
            args:  Vec::new(),
            span:  pat.span,
        })
    }

    fn paren_pat(&self, scope: &mut Scope, pat: ast::ParenPat) -> Pat {
        self.pat(scope, *pat.pat)
    }

    fn error_pat(&self, _scope: &mut Scope, pat: ast::ErrorPat) -> Pat {
        Pat::Error(ErrorPat { span: pat.span })
    }
}
