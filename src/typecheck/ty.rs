use crate::{
    rename,
    typecheck::{Arity, Kind, State},
    var::{Ident, Var},
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Ty {
    Io,
    Str,
    Int,
    Real,
    Lam,
    App(Box<Ty>, Box<Ty>),
    Var(Var),
    Type(Var),
}

impl Ty {
    pub fn reduce(&self) -> (&Ty, Vec<&Ty>) {
        match self {
            Ty::App(lambda, input) => {
                let (ty, mut args) = lambda.reduce();
                args.push(input);
                (ty, args)
            }

            _ => (self, Vec::new()),
        }
    }

    pub fn app(lambda: Ty, input: Ty) -> Ty {
        Ty::App(Box::new(lambda), Box::new(input))
    }

    pub fn lam(input: Ty, output: Ty) -> Ty {
        Ty::App(
            Box::new(Ty::App(
                Box::new(Ty::Lam),
                Box::new(input),
            )),
            Box::new(output),
        )
    }
}

impl State<'_> {
    pub fn ty(&mut self, ty: rename::Ty) -> Ty {
        match ty {
            rename::Ty::Var(ty) => self.var_ty(ty),
            rename::Ty::Lam(ty) => self.lam_ty(ty),
            rename::Ty::App(ty) => self.app_ty(ty),
            rename::Ty::Type(ty) => self.type_ty(ty),
            rename::Ty::Prim(ty) => self.prim_ty(ty),
            rename::Ty::Where(ty) => self.where_ty(ty),
            rename::Ty::Error(ty) => self.error_ty(ty),
        }
    }

    fn var_ty(&mut self, ty: rename::VarTy) -> Ty {
        Ty::Var(ty.var)
    }

    fn lam_ty(&mut self, ty: rename::LamTy) -> Ty {
        let input_span = ty.input.span();
        let output_span = ty.output.span();

        let input = self.ty(*ty.input);
        let output = self.ty(*ty.output);

        self.require_arity(&input, Arity::exact(0), input_span);
        self.require_arity(&output, Arity::exact(0), output_span);

        Ty::lam(input, output)
    }

    fn app_ty(&mut self, ty: rename::AppTy) -> Ty {
        let lambda = self.ty(*ty.lambda);
        let input = self.ty(*ty.input);

        self.require(
            &lambda,
            &Kind::arity(Arity::min(1)),
            ty.span,
        );

        Ty::app(lambda, input)
    }

    fn type_ty(&mut self, ty: rename::TypeTy) -> Ty {
        Ty::Type(ty.var)
    }

    fn prim_ty(&mut self, ty: rename::PrimTy) -> Ty {
        match ty.prim {
            rename::TyPrim::Io => Ty::Io,
            rename::TyPrim::Str => Ty::Str,
            rename::TyPrim::Int => Ty::Int,
            rename::TyPrim::Real => Ty::Real,
        }
    }

    fn where_ty(&mut self, ty: rename::WhereTy) -> Ty {
        let input = self.ty(*ty.input);
        let target = self.ty(*ty.target);
        let kind = self.kind(ty.kind);

        self.require(&target, &kind, ty.span);

        input
    }

    fn error_ty(&mut self, ty: rename::ErrorTy) -> Ty {
        Ty::Var(Var::new(Ident::new("{error}", ty.span)))
    }
}
