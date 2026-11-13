use crate::{
    diagnostic::Span,
    primitive::{BinPrim, PrimNum, UnPrim},
    rename::{
        AppTy, Def, Expr, ExprPrim, LamTy, PrimExpr, PrimTy, Scope, ScopeValue, State, Ty, TyPrim,
        VarExpr, VarTy, Vis,
    },
    var::{Ident, Var},
};

use super::{BindPat, LamExpr, Pat, ScopeValueKind};

impl State<'_> {
    pub(super) fn intrinsic(&mut self) -> Scope {
        let mut scope = Scope::new();

        self.add_io_intrinsics(&mut scope);
        self.add_num_intrinsics(&mut scope);

        scope
    }

    fn add_io_intrinsics(&mut self, scope: &mut Scope) {
        self.add_io_bind(scope);
        self.add_io_pure(scope);
    }

    fn add_io_bind(&mut self, scope: &mut Scope) {
        let a = Var::unnamed();
        let b = Var::unnamed();

        self.add_bin(
            scope,
            "io-bind",
            app_ty(prim_ty(TyPrim::Io), var_ty(&a)),
            lam_ty(
                var_ty(&a),
                app_ty(prim_ty(TyPrim::Io), var_ty(&b)),
            ),
            app_ty(prim_ty(TyPrim::Io), var_ty(&b)),
            BinPrim::IoBind,
        );
    }

    fn add_io_pure(&mut self, scope: &mut Scope) {
        let a = Var::unnamed();

        self.add_un(
            scope,
            "io-pure",
            var_ty(&a),
            app_ty(prim_ty(TyPrim::Io), var_ty(&a)),
            UnPrim::IoPure,
        );
    }

    fn add_num_intrinsics(&mut self, scope: &mut Scope) {
        self.add_int_intrinsics(scope);
        self.add_real_intrinsics(scope);
    }

    fn add_int_intrinsics(&mut self, scope: &mut Scope) {
        self.add_int_bin(scope, "int-add", BinPrim::Add);
        self.add_int_bin(scope, "int-sub", BinPrim::Sub);
        self.add_int_bin(scope, "int-mul", BinPrim::Mul);
        self.add_int_bin(scope, "int-div", BinPrim::Div);
    }

    fn add_real_intrinsics(&mut self, scope: &mut Scope) {
        self.add_real_bin(scope, "real-add", BinPrim::Add);
        self.add_real_bin(scope, "real-sub", BinPrim::Sub);
        self.add_real_bin(scope, "real-mul", BinPrim::Mul);
        self.add_real_bin(scope, "real-div", BinPrim::Div);
    }

    fn add_int_bin(&mut self, scope: &mut Scope, name: &str, prim: impl Fn(PrimNum) -> BinPrim) {
        self.add_num_bin(scope, name, PrimNum::Int, prim);
    }

    fn add_real_bin(&mut self, scope: &mut Scope, name: &str, prim: impl Fn(PrimNum) -> BinPrim) {
        self.add_num_bin(scope, name, PrimNum::Real, prim);
    }

    fn add_num_bin(
        &mut self,
        scope: &mut Scope,
        name: &str,
        num: PrimNum,
        prim: impl Fn(PrimNum) -> BinPrim,
    ) {
        let ty = match num {
            PrimNum::Int => TyPrim::Int,
            PrimNum::Real => TyPrim::Real,
        };

        self.add_prim_bin(scope, name, ty, ty, ty, prim(num));
    }

    fn add_prim_bin(
        &mut self,
        scope: &mut Scope,
        name: &str,
        lhs: TyPrim,
        rhs: TyPrim,
        out: TyPrim,
        prim: BinPrim,
    ) {
        self.add_bin(
            scope,
            name,
            prim_ty(lhs),
            prim_ty(rhs),
            prim_ty(out),
            prim,
        );
    }

    fn add_un(&mut self, scope: &mut Scope, name: &str, input: Ty, output: Ty, prim: UnPrim) {
        let var = Var::new(Ident::new(name, Span::DUMMY));

        let value = ScopeValue {
            vis:  Vis::Public,
            var:  var.clone(),
            kind: ScopeValueKind::Def,
        };

        scope.values.push(value);

        let input_var = Var::unnamed();

        let def = Def {
            ty:   Some(lam_ty(input, output)),
            var:  var.clone(),
            expr: lam_expr(
                &input_var,
                un_prim_expr(&input_var, prim),
            ),
            span: Span::DUMMY,
        };

        self.defs.insert(var.unique(), def);
    }

    fn add_bin(&mut self, scope: &mut Scope, name: &str, lhs: Ty, rhs: Ty, out: Ty, prim: BinPrim) {
        let var = Var::new(Ident::new(name, Span::DUMMY));

        let value = ScopeValue {
            vis:  Vis::Public,
            var:  var.clone(),
            kind: ScopeValueKind::Def,
        };

        scope.values.push(value);

        let lhs_var = Var::unnamed();
        let rhs_var = Var::unnamed();

        let def = Def {
            ty:   Some(lam_ty(lhs, lam_ty(rhs, out))),
            var:  var.clone(),
            expr: lam_expr(
                &lhs_var,
                lam_expr(
                    &rhs_var,
                    bin_prim_expr(&lhs_var, &rhs_var, prim),
                ),
            ),
            span: Span::DUMMY,
        };

        self.defs.insert(var.unique(), def);
    }
}

fn var_ty(var: &Var) -> Ty {
    Ty::Var(VarTy {
        var:  var.clone(),
        span: Span::DUMMY,
    })
}

fn app_ty(lambda: Ty, input: Ty) -> Ty {
    Ty::App(AppTy {
        lambda: Box::new(lambda),
        input:  Box::new(input),
        span:   Span::DUMMY,
    })
}

fn lam_ty(input: Ty, output: Ty) -> Ty {
    Ty::Lam(LamTy {
        input:  Box::new(input),
        output: Box::new(output),
        span:   Span::DUMMY,
    })
}

fn prim_ty(prim: TyPrim) -> Ty {
    Ty::Prim(PrimTy {
        prim,
        span: Span::DUMMY,
    })
}

fn un_prim_expr(input: &Var, prim: UnPrim) -> Expr {
    Expr::Prim(PrimExpr {
        prim: ExprPrim::Un(prim, Box::new(var_expr(input))),
        span: Span::DUMMY,
    })
}

fn bin_prim_expr(lhs: &Var, rhs: &Var, prim: BinPrim) -> Expr {
    Expr::Prim(PrimExpr {
        prim: ExprPrim::Bin(
            prim,
            Box::new(var_expr(lhs)),
            Box::new(var_expr(rhs)),
        ),
        span: Span::DUMMY,
    })
}

fn lam_expr(arg: &Var, output: Expr) -> Expr {
    Expr::Lam(LamExpr {
        arg:    Pat::Bind(BindPat {
            var:  Some(arg.clone()),
            span: Span::DUMMY,
        }),
        output: Box::new(output),
        span:   Span::DUMMY,
    })
}

fn var_expr(var: &Var) -> Expr {
    Expr::Var(VarExpr {
        var:  var.clone(),
        span: Span::DUMMY,
    })
}
