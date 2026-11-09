use crate::{rename, typecheck::State, var::Var};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Ty {
    Var(Var),
    Cons(Cons, Vec<Ty>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Cons {
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
            rename::Ty::Where(_ty) => todo!(),
            rename::Ty::Error(_ty) => todo!(),
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
}
