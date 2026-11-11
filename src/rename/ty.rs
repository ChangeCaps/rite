use crate::{
    ast,
    diagnostic::Span,
    rename::{Kind, Scope, State},
    var::Var,
};

use super::ScopeTyKind;

pub enum Ty {
    Var(VarTy),
    Lam(LamTy),
    App(AppTy),
    Prim(PrimTy),
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

pub struct PrimTy {
    pub prim: TyPrim,
    pub span: Span,
}

#[derive(Clone, Copy, Debug)]
pub enum TyPrim {
    Str,
    Int,
    Real,
}

pub struct WhereTy {
    pub input:  Box<Ty>,
    pub target: Box<Ty>,
    pub kind:   Kind,
    pub span:   Span,
}

pub struct ErrorTy {
    pub span: Span,
}

impl Ty {
    pub fn span(&self) -> Span {
        match self {
            Ty::Var(ty) => ty.span,
            Ty::Lam(ty) => ty.span,
            Ty::App(ty) => ty.span,
            Ty::Prim(ty) => ty.span,
            Ty::Where(ty) => ty.span,
            Ty::Error(ty) => ty.span,
        }
    }
}

impl State<'_> {
    pub fn ty(&self, scope: &mut Scope, ty: ast::Ty) -> Ty {
        match ty {
            ast::Ty::Var(ty) => self.var_ty(scope, ty),
            ast::Ty::Lam(ty) => self.lam_ty(scope, ty),
            ast::Ty::App(ty) => self.app_ty(scope, ty),
            ast::Ty::Where(ty) => self.where_ty(scope, ty),
            ast::Ty::Error(ty) => self.error_ty(scope, ty),
        }
    }

    fn var_ty(&self, scope: &mut Scope, ty: ast::VarTy) -> Ty {
        match self.find_ty(scope, ty.var.as_deref()) {
            Some(found) => match found.kind {
                ScopeTyKind::Var(ref var) => Ty::Var(VarTy {
                    var:  var.clone(),
                    span: ty.span,
                }),

                ScopeTyKind::Prim(ref prim) => Ty::Prim(PrimTy {
                    prim: *prim,
                    span: ty.span,
                }),
            },
            None => {
                let var = Var::from(ty.var);

                scope.tys.push(var.clone().into());

                Ty::Var(VarTy { var, span: ty.span })
            }
        }
    }

    fn lam_ty(&self, scope: &mut Scope, ty: ast::LamTy) -> Ty {
        let input = self.ty(scope, *ty.input);
        let output = self.ty(scope, *ty.output);

        Ty::Lam(LamTy {
            input:  Box::new(input),
            output: Box::new(output),
            span:   ty.span,
        })
    }

    fn app_ty(&self, scope: &mut Scope, ty: ast::AppTy) -> Ty {
        let lambda = self.ty(scope, *ty.lambda);
        let input = self.ty(scope, *ty.input);

        Ty::App(AppTy {
            lambda: Box::new(lambda),
            input:  Box::new(input),
            span:   ty.span,
        })
    }

    fn where_ty(&self, scope: &mut Scope, ty: ast::WhereTy) -> Ty {
        let input = self.ty(scope, *ty.input);
        let target = self.ty(scope, *ty.target);
        let kind = self.kind(scope, ty.kind);

        Ty::Where(WhereTy {
            input: Box::new(input),
            target: Box::new(target),
            kind,
            span: ty.span,
        })
    }

    fn error_ty(&self, _scope: &mut Scope, ty: ast::ErrorTy) -> Ty {
        Ty::Error(ErrorTy { span: ty.span })
    }
}
