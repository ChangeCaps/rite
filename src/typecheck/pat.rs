use crate::{
    diagnostic::Span,
    rename,
    typecheck::{State, Ty},
    var::Var,
};

#[derive(Debug)]
pub enum Pat {
    Str(StrPat),
    Int(IntPat),
    Bind(BindPat),
    Cons(ConsPat),
    Error(ErrorPat),
}

#[derive(Debug)]
pub struct StrPat {
    pub str:  String,
    pub span: Span,
}

#[derive(Debug)]
pub struct IntPat {
    pub int:  i64,
    pub span: Span,
}

#[derive(Debug)]
pub struct BindPat {
    pub var:  Option<Var>,
    pub span: Span,
}

#[derive(Debug)]
pub struct ConsPat {
    pub var:  Var,
    pub args: Vec<Pat>,
    pub span: Span,
}

#[derive(Debug)]
pub struct ErrorPat {
    pub span: Span,
}

impl State<'_> {
    pub fn pat(&mut self, pat: rename::Pat, ty: &Ty) -> Pat {
        match pat {
            rename::Pat::Str(pat) => self.str_pat(pat, ty),
            rename::Pat::Int(pat) => self.int_pat(pat, ty),
            rename::Pat::Bind(pat) => self.bind_pat(pat, ty),
            rename::Pat::Cons(pat) => self.cons_pat(pat, ty),
            rename::Pat::Error(pat) => self.error_pat(pat),
        }
    }

    fn str_pat(&mut self, pat: rename::StrPat, ty: &Ty) -> Pat {
        self.unify(ty, &Ty::Str, pat.span);

        Pat::Str(StrPat {
            str:  pat.str,
            span: pat.span,
        })
    }

    fn int_pat(&mut self, pat: rename::IntPat, ty: &Ty) -> Pat {
        self.unify(ty, &Ty::Int, pat.span);

        Pat::Int(IntPat {
            int:  pat.int,
            span: pat.span,
        })
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
