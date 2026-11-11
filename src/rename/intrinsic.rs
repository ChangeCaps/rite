use crate::{
    diagnostic::Span,
    primitive::{BinPrim, PrimNum},
    rename::{
        Def, Expr, ExprPrim, LamTy, PrimExpr, PrimTy, Scope, ScopeValue, State, Ty, TyPrim,
        VarExpr, Vis,
    },
    var::{Ident, Var},
};

use super::{BindPat, LamExpr, Pat, ScopeValueKind};

impl State<'_> {
    pub(super) fn intrinsic(&mut self) -> Scope {
        let mut scope = Scope::new();

        self.add_num_prims(&mut scope);

        scope
    }

    fn add_num_prims(&mut self, scope: &mut Scope) {
        self.add_int_prims(scope);
        self.add_real_prims(scope);
    }

    fn add_int_prims(&mut self, scope: &mut Scope) {
        self.add_int_bin(scope, "int-add", BinPrim::Add);
        self.add_int_bin(scope, "int-sub", BinPrim::Sub);
        self.add_int_bin(scope, "int-mul", BinPrim::Mul);
        self.add_int_bin(scope, "int-div", BinPrim::Div);
    }

    fn add_real_prims(&mut self, scope: &mut Scope) {
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
            ty:   Some(lam_ty(
                prim_ty(lhs),
                lam_ty(prim_ty(rhs), prim_ty(out)),
            )),
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
