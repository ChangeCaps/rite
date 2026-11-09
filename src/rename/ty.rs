use crate::{
    ast,
    diagnostic::Span,
    rename::{Kind, State},
    var::Var,
};

pub enum Ty {
    Var(VarTy),
    Lam(LamTy),
    App(AppTy),
    Where(WhereTy),
    Error(ErrorTy),
}

pub struct VarTy {
    pub var:  Var,
    pub span: Span,
}

pub struct LamTy {
    pub input:  Box<Ty>,
    pub output: Box<Ty>,
    pub span:   Span,
}

pub struct AppTy {
    pub lambda: Box<Ty>,
    pub input:  Box<Ty>,
    pub span:   Span,
}

pub struct WhereTy {
    pub ty:    Box<Ty>,
    pub input: Box<Ty>,
    pub kind:  Kind,
    pub span:  Span,
}

pub struct ErrorTy {
    pub span: Span,
}

impl State<'_> {
    pub fn ty(&mut self, ty: ast::Ty) -> Ty {
        match ty {
            ast::Ty::Var(ty) => self.var_ty(ty),
            ast::Ty::Lam(ty) => self.lam_ty(ty),
            ast::Ty::App(ty) => self.app_ty(ty),
            ast::Ty::Where(ty) => self.where_ty(ty),
            ast::Ty::Error(ty) => self.error_ty(ty),
        }
    }

    fn var_ty(&mut self, ty: ast::VarTy) -> Ty {
        match self.find_ty(ty.var.as_ref()) {
            Some(var) => Ty::Var(VarTy {
                var:  var.clone(),
                span: ty.span,
            }),

            None => {
                let var = Var::from(ty.var);

                self.tys.push(var.clone());

                Ty::Var(VarTy { var, span: ty.span })
            }
        }
    }

    fn lam_ty(&mut self, ty: ast::LamTy) -> Ty {
        let input = self.ty(*ty.input);
        let output = self.ty(*ty.output);

        Ty::Lam(LamTy {
            input:  Box::new(input),
            output: Box::new(output),
            span:   ty.span,
        })
    }

    fn app_ty(&mut self, ty: ast::AppTy) -> Ty {
        let lambda = self.ty(*ty.lambda);
        let input = self.ty(*ty.input);

        Ty::App(AppTy {
            lambda: Box::new(lambda),
            input:  Box::new(input),
            span:   ty.span,
        })
    }

    fn where_ty(&mut self, where_ty: ast::WhereTy) -> Ty {
        let ty = self.ty(*where_ty.ty);
        let input = self.ty(*where_ty.input);
        let kind = self.kind(where_ty.kind);

        Ty::Where(WhereTy {
            ty: Box::new(ty),
            input: Box::new(input),
            kind,
            span: where_ty.span,
        })
    }

    fn error_ty(&mut self, ty: ast::ErrorTy) -> Ty {
        Ty::Error(ErrorTy { span: ty.span })
    }
}
