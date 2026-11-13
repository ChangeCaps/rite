use std::collections::HashMap;

use crate::{
    diagnostic::{Diagnostic, Span},
    primitive::{BinPrim, UnPrim},
    rename,
    typecheck::{Kind, Pat, State, Ty, TypeCons, exhaust},
    var::{Ident, Var},
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
    Rec(RecExpr),
    With(WithExpr),
    Prim(PrimExpr),
    Field(FieldExpr),
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

pub struct RecExpr {
    pub type_:  Var,
    pub fields: Vec<ExprField>,
    pub ty:     Ty,
    pub span:   Span,
}

pub struct WithExpr {
    pub input:  Box<Expr>,
    pub fields: Vec<ExprField>,
    pub ty:     Ty,
    pub span:   Span,
}

pub struct ExprField {
    pub name: Ident,
    pub expr: Expr,
    pub span: Span,
}

pub struct PrimExpr {
    pub prim: ExprPrim,
    pub ty:   Ty,
    pub span: Span,
}

pub enum ExprPrim {
    Un(UnPrim, Box<Expr>),
    Bin(BinPrim, Box<Expr>, Box<Expr>),
}

pub struct FieldExpr {
    pub input: Box<Expr>,
    pub name:  Ident,
    pub ty:    Ty,
    pub span:  Span,
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
            Expr::Rec(expr) => expr.span,
            Expr::With(expr) => expr.span,
            Expr::Prim(expr) => expr.span,
            Expr::Field(expr) => expr.span,
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
            Expr::Rec(expr) => &expr.ty,
            Expr::With(expr) => &expr.ty,
            Expr::Prim(expr) => &expr.ty,
            Expr::Field(expr) => &expr.ty,
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
            rename::Expr::Rec(expr) => self.rec_expr_down(expr),
            rename::Expr::With(expr) => self.with_expr_down(expr),
            rename::Expr::Prim(expr) => self.prim_expr_down(expr),
            rename::Expr::Field(expr) => self.field_expr_down(expr),
            rename::Expr::Match(expr) => self.match_expr_down(expr),
            rename::Expr::Error(expr) => self.error_expr_down(expr),
        }
    }

    fn lit_expr_down(&mut self, expr: rename::LitExpr) -> Expr {
        let ty = match expr.lit {
            rename::Lit::Str(_) => Ty::Str,
            rename::Lit::Int(_) => Ty::Int,
            rename::Lit::Real(_) => Ty::Real,
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
        let ty = Ty::Var(expr.var.clone());

        Expr::Var(VarExpr {
            var: expr.var.clone(),
            ty,
            span: expr.span,
        })
    }

    fn def_expr_down(&mut self, expr: rename::DefExpr) -> Expr {
        let mut ty = Ty::Var(expr.var.clone());

        if expr.ascribed {
            self.instantiate(&mut HashMap::new(), &mut ty);
        }

        Expr::Def(DefExpr {
            var: expr.var.clone(),
            ty,
            span: expr.span,
        })
    }

    fn cons_expr_down(&mut self, expr: rename::ConsExpr) -> Expr {
        let mut ty = Ty::Var(expr.var.clone());
        self.instantiate(&mut HashMap::new(), &mut ty);

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
        let mut subst = HashMap::from([(trait_.input.unique(), input.clone())]);

        let mut ty = Ty::Var(expr.var.clone());
        self.instantiate(&mut subst, &mut ty);

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

        let ty = Ty::lam(input, output.ty().clone());

        Expr::Lam(LamExpr {
            arg,
            output: Box::new(output),
            ty,
            span: expr.span,
        })
    }

    fn app_expr_down(&mut self, expr: rename::AppExpr) -> Expr {
        let input = self.expr_down(*expr.input);
        let output = Ty::Var(Var::unnamed());
        let lambda = Ty::lam(input.ty().clone(), output.clone());
        let lambda = self.expr_up(*expr.lambda, lambda);

        Expr::App(AppExpr {
            lambda: Box::new(lambda),
            input:  Box::new(input),
            ty:     output,
            span:   expr.span,
        })
    }

    fn rec_expr_down(&mut self, expr: rename::RecExpr) -> Expr {
        let mut subst = HashMap::new();
        let mut ty = self.tycx.type_ty(expr.type_.clone());
        self.instantiate(&mut subst, &mut ty);

        let TypeCons::Record(ref cons) = self.tycx.get_type(&expr.type_).cons else {
            panic!();
        };

        let fields = cons.fields.clone();
        let fields = expr
            .fields
            .into_iter()
            .map(|field| {
                let expr = self.expr_down(field.expr);
                let mut ty = fields
                    .iter()
                    .find(|f| f.name == field.name)
                    .map(|f| f.ty.clone())
                    .expect("field should be present");

                self.instantiate(&mut subst, &mut ty);
                self.unify(expr.ty(), &ty, expr.span());

                ExprField {
                    name: field.name,
                    expr,
                    span: field.span,
                }
            })
            .collect();

        Expr::Rec(RecExpr {
            type_: expr.type_,
            fields,
            ty,
            span: expr.span,
        })
    }

    fn with_expr_down(&mut self, expr: rename::WithExpr) -> Expr {
        let input = self.expr_down(*expr.input);
        let ty = input.ty().clone();

        let fields = expr
            .fields
            .into_iter()
            .map(|field| {
                let expr = self.expr_down(field.expr);

                let kind = Kind::field(
                    field.name.to_string(),
                    expr.ty().clone(),
                );

                self.require(&ty, &kind, field.span);

                ExprField {
                    name: field.name,
                    expr,
                    span: field.span,
                }
            })
            .collect();

        Expr::With(WithExpr {
            input: Box::new(input),
            fields,
            ty,
            span: expr.span,
        })
    }

    fn prim_expr_down(&mut self, expr: rename::PrimExpr) -> Expr {
        let prim = match expr.prim {
            rename::ExprPrim::Un(prim, input) => {
                let input = self.expr_down(*input);

                ExprPrim::Un(prim, Box::new(input))
            }

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

    fn field_expr_down(&mut self, expr: rename::FieldExpr) -> Expr {
        let ty = Ty::Var(Var::unnamed());

        let Some(name) = expr.name else {
            return Expr::Error(ErrorExpr {
                ty,
                span: expr.span,
            });
        };

        let input = self.expr_down(*expr.input);

        self.require(
            input.ty(),
            &Kind::field(name.to_string(), ty.clone()),
            expr.span,
        );

        Expr::Field(FieldExpr {
            input: Box::new(input),
            name,
            ty,
            span: expr.span,
        })
    }

    fn match_expr_down(&mut self, expr: rename::MatchExpr) -> Expr {
        let input = self.expr_down(*expr.input);
        let ty = Ty::Var(Var::unnamed());

        let mut matrix = exhaust::Matrix::new();

        let arms = expr
            .arms
            .into_iter()
            .filter_map(|arm| {
                let row = self.row(&arm.pat);
                let span = arm.pat.span();

                let pat = self.pat(arm.pat, input.ty());
                let output = self.expr_down(arm.output);

                self.unify(&ty, output.ty(), arm.span);

                if self.is_useful(matrix.clone(), row.clone()) {
                    matrix.push(row);

                    Some(MatchArm {
                        pat,
                        output,
                        span: arm.span,
                    })
                } else {
                    let diagnostic = Diagnostic::warn("unreachable pattern")
                        .label(span, "no value can reach this");

                    self.reporter.report(diagnostic);

                    None
                }
            })
            .collect();

        let pats = self.exhaust(matrix);

        if !pats.is_empty() {
            let fmts = pats
                .iter()
                .take(3)
                .map(|pat| format!("`{pat}`"))
                .collect::<Vec<_>>();

            let diagnostic = if pats.len() > 3 {
                Diagnostic::error("non-exhaustive patterns").label(
                    expr.span,
                    format!(
                        "patterns: {} and {} more not covered",
                        fmts.join(", "),
                        pats.len(),
                    ),
                )
            } else {
                Diagnostic::error("non-exhaustive patterns").label(
                    expr.span,
                    format!(
                        "patterns: {} not covered",
                        fmts.join(", "),
                    ),
                )
            };

            self.reporter.report(diagnostic);
        }

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
