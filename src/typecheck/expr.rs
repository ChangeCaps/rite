use std::collections::HashMap;

use crate::{
    diagnostic::Span,
    primitive::BinPrim,
    rename,
    typecheck::{Cons, Pat, State, Ty},
    var::Var,
};

pub enum Expr {
    Lit(LitExpr),
    Var(VarExpr),
    Def(DefExpr),
    Cons(ConsExpr),
    Method(MethodExpr),
    Let(LetExpr),
    Lam(LamExpr),
    App(AppExpr),
    Prim(PrimExpr),
    Match(MatchExpr),
    Error(ErrorExpr),
}

pub struct LitExpr {
    pub lit:  Lit,
    pub ty:   Ty,
    pub span: Span,
}

pub enum Lit {
    Str(String),
    Int(i64),
    Real(f64),
}

pub struct VarExpr {
    pub var:  Var,
    pub ty:   Ty,
    pub span: Span,
}

pub struct DefExpr {
    pub var:  Var,
    pub ty:   Ty,
    pub span: Span,
}

pub struct ConsExpr {
    pub var:   Var,
    pub type_: Var,
    pub ty:    Ty,
    pub span:  Span,
}

pub struct MethodExpr {
    pub var:    Var,
    pub trait_: Var,
    pub input:  Ty,
    pub ty:     Ty,
    pub span:   Span,
}

pub struct LetExpr {
    pub pat:    Pat,
    pub input:  Box<Expr>,
    pub output: Box<Expr>,
    pub ty:     Ty,
    pub span:   Span,
}

pub struct LamExpr {
    pub arg:    Pat,
    pub output: Box<Expr>,
    pub ty:     Ty,
    pub span:   Span,
}

pub struct AppExpr {
    pub lambda: Box<Expr>,
    pub input:  Box<Expr>,
    pub ty:     Ty,
    pub span:   Span,
}

pub struct PrimExpr {
    pub prim: ExprPrim,
    pub ty:   Ty,
    pub span: Span,
}

pub enum ExprPrim {
    Bin(BinPrim, Box<Expr>, Box<Expr>),
}

pub struct MatchExpr {
    pub input: Box<Expr>,
    pub arms:  Vec<MatchArm>,
    pub ty:    Ty,
    pub span:  Span,
}

pub struct MatchArm {
    pub pat:    Pat,
    pub output: Expr,
    pub span:   Span,
}

pub struct ErrorExpr {
    pub ty:   Ty,
    pub span: Span,
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Lit(expr) => expr.span,
            Expr::Let(expr) => expr.span,
            Expr::Var(expr) => expr.span,
            Expr::Def(expr) => expr.span,
            Expr::Cons(expr) => expr.span,
            Expr::Method(expr) => expr.span,
            Expr::Lam(expr) => expr.span,
            Expr::App(expr) => expr.span,
            Expr::Prim(expr) => expr.span,
            Expr::Match(expr) => expr.span,
            Expr::Error(expr) => expr.span,
        }
    }

    pub fn ty(&self) -> &Ty {
        match self {
            Expr::Lit(expr) => &expr.ty,
            Expr::Let(expr) => &expr.ty,
            Expr::Var(expr) => &expr.ty,
            Expr::Def(expr) => &expr.ty,
            Expr::Cons(expr) => &expr.ty,
            Expr::Method(expr) => &expr.ty,
            Expr::Lam(expr) => &expr.ty,
            Expr::App(expr) => &expr.ty,
            Expr::Prim(expr) => &expr.ty,
            Expr::Match(expr) => &expr.ty,
            Expr::Error(expr) => &expr.ty,
        }
    }
}

impl State<'_> {
    pub fn expr_up(&mut self, expr: rename::Expr, ty: Ty) -> Expr {
        let actual = self.expr_down(expr);
        self.unify(&ty, actual.ty(), actual.span());
        actual
    }

    pub fn expr_down(&mut self, expr: rename::Expr) -> Expr {
        match expr {
            rename::Expr::Lit(expr) => self.lit_expr_down(expr),
            rename::Expr::Let(expr) => self.let_expr_down(expr),
            rename::Expr::Var(expr) => self.var_expr_down(expr),
            rename::Expr::Def(expr) => self.def_expr_down(expr),
            rename::Expr::Cons(expr) => self.cons_expr_down(expr),
            rename::Expr::Method(expr) => self.method_expr_down(expr),
            rename::Expr::Lam(expr) => self.lam_expr_down(expr),
            rename::Expr::App(expr) => self.app_expr_down(expr),
            rename::Expr::Prim(expr) => self.prim_expr_down(expr),
            rename::Expr::Match(expr) => self.match_expr_down(expr),
            rename::Expr::Error(expr) => self.error_expr_down(expr),
        }
    }

    fn lit_expr_down(&mut self, expr: rename::LitExpr) -> Expr {
        let ty = match expr.lit {
            rename::Lit::Str(_) => Ty::Cons(Cons::Str, Vec::new()),
            rename::Lit::Int(_) => Ty::Cons(Cons::Int, Vec::new()),
            rename::Lit::Real(_) => Ty::Cons(Cons::Real, Vec::new()),
        };

        let lit = match expr.lit {
            rename::Lit::Str(str) => Lit::Str(str),
            rename::Lit::Int(int) => Lit::Int(int),
            rename::Lit::Real(real) => Lit::Real(real),
        };

        Expr::Lit(LitExpr {
            lit,
            ty,
            span: expr.span,
        })
    }

    fn let_expr_down(&mut self, expr: rename::LetExpr) -> Expr {
        let input = self.expr_down(*expr.input);

        let pat = self.pat(expr.pat, input.ty());

        let output = self.expr_down(*expr.output);
        let ty = output.ty().clone();

        Expr::Let(LetExpr {
            pat,
            input: Box::new(input),
            output: Box::new(output),
            ty,
            span: expr.span,
        })
    }

    fn var_expr_down(&mut self, expr: rename::VarExpr) -> Expr {
        let ty = self.get_ty_var(expr.var.clone());

        Expr::Var(VarExpr {
            var: expr.var.clone(),
            ty,
            span: expr.span,
        })
    }

    fn def_expr_down(&mut self, expr: rename::DefExpr) -> Expr {
        let mut ty = self.get_ty_var(expr.var.clone());

        if expr.ascribed {
            ty = self.instantiate(HashMap::new(), ty);
        }

        Expr::Def(DefExpr {
            var: expr.var.clone(),
            ty,
            span: expr.span,
        })
    }

    fn cons_expr_down(&mut self, expr: rename::ConsExpr) -> Expr {
        let ty = self.get_ty_var(expr.var.clone());
        let ty = self.instantiate(HashMap::new(), ty);

        Expr::Cons(ConsExpr {
            var: expr.var.clone(),
            type_: expr.type_,
            ty,
            span: expr.span,
        })
    }

    fn method_expr_down(&mut self, expr: rename::MethodExpr) -> Expr {
        let trait_ = self.tycx.get_trait(&expr.trait_);

        // create new instance of input with the correct kind
        let input = Var::unnamed();
        if let Some(kind) = self.kinds.get(&trait_.input.unique()) {
            self.kinds.insert(input.unique(), kind.clone());
        }

        let input = Ty::Var(input);

        let subst = HashMap::from([(trait_.input.unique(), input.clone())]);

        let ty = self.get_ty_var(expr.var.clone());
        let ty = self.instantiate(subst, ty);

        Expr::Method(MethodExpr {
            var: expr.var.clone(),
            trait_: expr.trait_,
            input,
            ty,
            span: expr.span,
        })
    }

    fn lam_expr_down(&mut self, expr: rename::LamExpr) -> Expr {
        let input = Ty::Var(Var::unnamed());

        let arg = self.pat(expr.arg, &input);
        let output = self.expr_down(*expr.output);

        let ty = Ty::Cons(
            Cons::Lam,
            vec![input, output.ty().clone()],
        );

        Expr::Lam(LamExpr {
            arg,
            output: Box::new(output),
            ty,
            span: expr.span,
        })
    }

    fn app_expr_down(&mut self, expr: rename::AppExpr) -> Expr {
        let input = self.expr_down(*expr.input);

        let output = self.get_ty_var(Var::unnamed());

        let lambda = Ty::Cons(
            Cons::Lam,
            vec![input.ty().clone(), output.clone()],
        );

        let lambda = self.expr_up(*expr.lambda, lambda);

        Expr::App(AppExpr {
            lambda: Box::new(lambda),
            input:  Box::new(input),
            ty:     output,
            span:   expr.span,
        })
    }

    fn prim_expr_down(&mut self, expr: rename::PrimExpr) -> Expr {
        let prim = match expr.prim {
            rename::ExprPrim::Bin(prim, lhs, rhs) => {
                let lhs = self.expr_down(*lhs);
                let rhs = self.expr_down(*rhs);

                ExprPrim::Bin(prim, Box::new(lhs), Box::new(rhs))
            }
        };

        let ty = Ty::Var(Var::unnamed());

        Expr::Prim(PrimExpr {
            prim,
            ty,
            span: expr.span,
        })
    }

    fn match_expr_down(&mut self, expr: rename::MatchExpr) -> Expr {
        let input = self.expr_down(*expr.input);
        let ty = Ty::Var(Var::unnamed());

        let arms = expr
            .arms
            .into_iter()
            .map(|arm| {
                let pat = self.pat(arm.pat, input.ty());
                let output = self.expr_down(arm.output);

                self.unify(&ty, output.ty(), arm.span);

                MatchArm {
                    pat,
                    output,
                    span: arm.span,
                }
            })
            .collect();

        Expr::Match(MatchExpr {
            input: Box::new(input),
            arms,
            ty,
            span: expr.span,
        })
    }

    fn error_expr_down(&mut self, expr: rename::ErrorExpr) -> Expr {
        Expr::Error(ErrorExpr {
            ty:   Ty::Var(Var::unnamed()),
            span: expr.span,
        })
    }
}
