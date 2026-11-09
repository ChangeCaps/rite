use std::collections::HashMap;

use crate::{
    diagnostic::Span,
    rename,
    typecheck::{Cons, State, Ty},
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
    pub fn pat(&mut self, pat: rename::Pat, ty: &Ty) -> Pat {
        match pat {
            rename::Pat::Bind(pat) => self.bind_pat(pat, ty),
            rename::Pat::Cons(pat) => self.cons_pat(pat, ty),
            rename::Pat::Error(pat) => self.error_pat(pat),
        }
    }

    fn bind_pat(&mut self, pat: rename::BindPat, ty: &Ty) -> Pat {
        if let Some(var) = pat.var.clone() {
            let actual = self.get_ty_var(var);
            self.unify(ty, &actual, pat.span);
        }

        Pat::Bind(BindPat {
            var:  pat.var.clone(),
            span: pat.span,
        })
    }

    fn cons_pat(&mut self, pat: rename::ConsPat, ty: &Ty) -> Pat {
        let actual = self.get_ty_var(pat.var.clone());
        let actual = self.instantiate(HashMap::new(), actual);

        let (args, actual) = pat.args.into_iter().fold(
            (Vec::new(), actual),
            |(mut pats, ty), arg| {
                let Ty::Cons(cons, mut args) = ty else {
                    panic!();
                };

                debug_assert_eq!(cons, Cons::Lam);

                let output = args.pop().unwrap();
                let ty = args.pop().unwrap();

                let pat = self.pat(arg, &ty);
                pats.push(pat);

                (pats, output)
            },
        );

        self.unify(ty, &actual, pat.span);

        Pat::Cons(ConsPat {
            var: pat.var,
            args,
            span: pat.span,
        })
    }

    fn error_pat(&mut self, pat: rename::ErrorPat) -> Pat {
        Pat::Error(ErrorPat { span: pat.span })
    }
}
