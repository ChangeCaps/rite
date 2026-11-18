use crate::{
    ast,
    diagnostic::{Diagnostic, Span},
    rename::{Kind, Scope, ScopeTy, State},
    var::Var,
};

use super::ScopeTyKind;

pub enum Ty {
    Var(VarTy),
    Lam(LamTy),
    App(AppTy),
    Type(TypeTy),
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

pub struct TypeTy {
    pub var:  Var,
    pub span: Span,
}

pub struct PrimTy {
    pub prim: TyPrim,
    pub span: Span,
}

#[derive(Clone, Copy, Debug)]
pub enum TyPrim {
    Io,
    Str,
    Int,
    Real,
}

pub struct WhereTy {
    pub input:  Box<Ty>,
    pub target: Box<Ty>,
    pub kinds:  Vec<Kind>,
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
            Ty::Type(ty) => ty.span,
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
            ast::Ty::Bin(ty) => self.bin_ty(scope, ty),
            ast::Ty::App(ty) => self.app_ty(scope, ty),
            ast::Ty::List(ty) => self.list_ty(scope, ty),
            ast::Ty::Paren(ty) => self.paren_ty(scope, ty),
            ast::Ty::Where(ty) => self.where_ty(scope, ty),
            ast::Ty::Error(ty) => self.error_ty(scope, ty),
        }
    }

    fn var_ty(&self, scope: &mut Scope, ty: ast::VarTy) -> Ty {
        if !ty.path.is_empty() {
            if let Some(scope) = self.find_module(scope, &ty.path) {
                if let Some(found) = scope.find_ty(&ty.var) {
                    return self.scope_ty(found, ty.span);
                } else {
                    let diagnostic =
                        Diagnostic::error("undefined type").label(ty.span, "found here");
                    self.reporter.report(diagnostic);
                }
            }

            return Ty::Error(ErrorTy { span: ty.span });
        }

        match self.find_ty(scope, &ty.var) {
            Some(found) => self.scope_ty(found, ty.span),

            None => {
                if let Some(c) = ty.var.as_str().chars().next()
                    && c.is_uppercase()
                {
                    let diagnostic = Diagnostic::warn("PascalCase type variable").label(
                        ty.span,
                        "type variables should use camelCase",
                    );

                    self.reporter.report(diagnostic);
                }

                let var = Var::new(ty.var);

                scope.tys.push(var.clone().into());

                Ty::Var(VarTy { var, span: ty.span })
            }
        }
    }

    fn scope_ty(&self, ty: &ScopeTy, span: Span) -> Ty {
        match ty.kind {
            ScopeTyKind::Var(ref var) => Ty::Var(VarTy {
                var: var.clone(),
                span,
            }),

            ScopeTyKind::Type(ref var) => Ty::Type(TypeTy {
                var: var.clone(),
                span,
            }),

            ScopeTyKind::Prim(ref prim) => Ty::Prim(PrimTy { prim: *prim, span }),
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

    fn bin_ty(&self, scope: &mut Scope, ty: ast::BinTy) -> Ty {
        let op = match self.find_ty(scope, ty.op.as_str()) {
            Some(found) => self.scope_ty(found, ty.span),

            None => {
                let diagnostic = Diagnostic::error("operator not defined").label(
                    ty.span,
                    format!("`{}` used here", ty.op.as_str()),
                );

                self.reporter.report(diagnostic);

                Ty::Error(ErrorTy { span: ty.span })
            }
        };

        let lhs = self.ty(scope, *ty.lhs);
        let rhs = self.ty(scope, *ty.rhs);

        Ty::App(AppTy {
            lambda: Box::new(Ty::App(AppTy {
                lambda: Box::new(op),
                input:  Box::new(lhs),
                span:   ty.span,
            })),
            input:  Box::new(rhs),
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

    fn list_ty(&self, scope: &mut Scope, ty: ast::ListTy) -> Ty {
        let op = match self.find_ty(scope, "[]") {
            Some(found) => self.scope_ty(found, ty.span),

            None => {
                let diagnostic =
                    Diagnostic::error("operator not defined").label(ty.span, "`[]` used here");

                self.reporter.report(diagnostic);

                Ty::Error(ErrorTy { span: ty.span })
            }
        };

        let item = self.ty(scope, *ty.item);

        Ty::App(AppTy {
            lambda: Box::new(op),
            input:  Box::new(item),
            span:   ty.span,
        })
    }

    fn paren_ty(&self, scope: &mut Scope, ty: ast::ParenTy) -> Ty {
        self.ty(scope, *ty.ty)
    }

    fn where_ty(&self, scope: &mut Scope, ty: ast::WhereTy) -> Ty {
        let input = self.ty(scope, *ty.input);
        let target = self.ty(scope, *ty.target);
        let kinds = ty
            .kinds
            .into_iter()
            .map(|kind| self.kind(scope, kind))
            .collect();

        Ty::Where(WhereTy {
            input: Box::new(input),
            target: Box::new(target),
            kinds,
            span: ty.span,
        })
    }

    fn error_ty(&self, _scope: &mut Scope, ty: ast::ErrorTy) -> Ty {
        Ty::Error(ErrorTy { span: ty.span })
    }
}
