use crate::{
    rename,
    typecheck::State,
    var::{Ident, Var},
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Ty {
    Var(Var),
    Cons(Cons, Vec<Ty>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Cons {
    Str,
    Int,
    Real,
    App,
    Lam,
    Type(Var, usize),
}

impl State<'_> {
    pub fn ty(&mut self, ty: rename::Ty) -> Ty {
        match ty {
            rename::Ty::Var(ty) => self.var_ty(ty),
            rename::Ty::Lam(ty) => self.lam_ty(ty),
            rename::Ty::App(ty) => self.app_ty(ty),
            rename::Ty::Prim(ty) => self.prim_ty(ty),
            rename::Ty::Where(ty) => self.where_ty(ty),
            rename::Ty::Error(ty) => self.error_ty(ty),
        }
    }

    fn var_ty(&mut self, ty: rename::VarTy) -> Ty {
        self.get_ty_var(ty.var)
    }

    fn lam_ty(&mut self, ty: rename::LamTy) -> Ty {
        let input = self.ty(*ty.input);
        let output = self.ty(*ty.output);

        Ty::Cons(Cons::Lam, vec![input, output])
    }

    fn app_ty(&mut self, ty: rename::AppTy) -> Ty {
        let lambda = self.ty(*ty.lambda);
        let input = self.ty(*ty.input);

        Ty::Cons(Cons::App, vec![lambda, input])
    }

    fn prim_ty(&mut self, _ty: rename::PrimTy) -> Ty {
        todo!()
    }

    fn where_ty(&mut self, ty: rename::WhereTy) -> Ty {
        let input = self.ty(*ty.input);
        let target = self.ty(*ty.target);
        let kind = self.kind(ty.kind);

        self.require_kind(&target, &kind, ty.span);

        input
    }

    fn error_ty(&mut self, ty: rename::ErrorTy) -> Ty {
        Ty::Var(Var::new(Ident::new("{error}", ty.span)))
    }
}
