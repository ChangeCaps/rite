use crate::{
    diagnostic::Span,
    rename,
    typecheck::{State, Ty},
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
            let actual = Ty::Var(var);
            self.unify(ty, &actual, pat.span);
        }

        Pat::Bind(BindPat {
            var:  pat.var.clone(),
            span: pat.span,
        })
    }

    fn cons_pat(&mut self, pat: rename::ConsPat, ty: &Ty) -> Pat {
        let cons = self.tycx.get_tuple_cons(&pat.type_, &pat.var);

        let mut subst = Default::default();
        let mut tys = cons.args.clone();

        self.instantiate_all(&mut subst, &mut tys);

        let args = pat
            .args
            .into_iter()
            .zip(tys.iter())
            .map(|(arg, ty)| self.pat(arg, ty))
            .collect();

        let mut type_ty = self.tycx.type_ty(pat.type_);
        self.instantiate(&mut subst, &mut type_ty);
        self.unify(ty, &type_ty, pat.span);

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
