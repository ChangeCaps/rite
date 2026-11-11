use crate::{
    ast,
    diagnostic::{Diagnostic, Span},
    primitive::BinPrim,
    rename::{Pat, Scope, ScopeValue, ScopeValueKind, State},
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
    pub span: Span,
}

pub enum Lit {
    Str(String),
    Int(i64),
    Real(f64),
}

pub struct VarExpr {
    pub var:  Var,
    pub span: Span,
}

pub struct DefExpr {
    pub var:      Var,
    pub ascribed: bool,
    pub span:     Span,
}

pub struct ConsExpr {
    pub var:   Var,
    pub type_: Var,
    pub span:  Span,
}

pub struct MethodExpr {
    pub var:    Var,
    pub trait_: Var,
    pub span:   Span,
}

pub struct LetExpr {
    pub pat:    Pat,
    pub input:  Box<Expr>,
    pub output: Box<Expr>,
    pub span:   Span,
}

pub struct LamExpr {
    pub arg:    Pat,
    pub output: Box<Expr>,
    pub span:   Span,
}

pub struct AppExpr {
    pub lambda: Box<Expr>,
    pub input:  Box<Expr>,
    pub span:   Span,
}

pub struct MatchExpr {
    pub input: Box<Expr>,
    pub arms:  Vec<MatchArm>,
    pub span:  Span,
}

pub struct MatchArm {
    pub pat:    Pat,
    pub output: Expr,
    pub span:   Span,
}

pub struct PrimExpr {
    pub prim: ExprPrim,
    pub span: Span,
}

pub enum ExprPrim {
    Bin(BinPrim, Box<Expr>, Box<Expr>),
}

pub struct ErrorExpr {
    pub span: Span,
}

impl State<'_> {
    pub fn expr_with_args(
        &mut self,
        mut scope: Scope,
        args: Vec<ast::Pat>,
        expr: ast::Expr,
    ) -> Expr {
        let args = args
            .into_iter()
            .map(|arg| self.pat(&mut scope, arg))
            .collect::<Vec<_>>();

        let output = self.expr(&scope, expr);

        args.into_iter().rfold(output, |expr, arg| {
            let span = arg.span();

            Expr::Lam(LamExpr {
                arg,
                output: Box::new(expr),
                span,
            })
        })
    }

    pub fn expr(&mut self, scope: &Scope, expr: ast::Expr) -> Expr {
        match expr {
            ast::Expr::Lit(expr) => self.lit_expr(scope, expr),
            ast::Expr::Let(expr) => self.let_expr(scope, expr),
            ast::Expr::Var(expr) => self.var_expr(scope, expr),
            ast::Expr::Lam(expr) => self.lam_expr(scope, expr),
            ast::Expr::App(expr) => self.app_expr(scope, expr),
            ast::Expr::Bin(expr) => self.bin_expr(scope, expr),
            ast::Expr::Seq(expr) => self.seq_expr(scope, expr),
            ast::Expr::Match(expr) => self.match_expr(scope, expr),
            ast::Expr::Error(expr) => self.error_expr(scope, expr),
        }
    }

    fn lit_expr(&mut self, _scope: &Scope, expr: ast::LitExpr) -> Expr {
        let lit = match expr.lit {
            ast::Lit::Str(str) => Lit::Str(str),
            ast::Lit::Int(int) => Lit::Int(int),
            ast::Lit::Real(real) => Lit::Real(real),
        };

        Expr::Lit(LitExpr {
            lit,
            span: expr.span,
        })
    }

    fn let_expr(&mut self, scope: &Scope, expr: ast::LetExpr) -> Expr {
        let input = self.expr(scope, *expr.input);

        let mut scope = scope.clone();
        let pat = self.pat(&mut scope, expr.pat);
        let output = self.expr(&scope, *expr.output);

        match expr.assign {
            ast::Assign::Assign => Expr::Let(LetExpr {
                pat,
                input: Box::new(input),
                output: Box::new(output),
                span: expr.span,
            }),

            ast::Assign::Bind => {
                let Some(operator) = self.find_value(&scope, Some(">>=")) else {
                    todo!()
                };

                let operator = self.value_expr(operator.clone(), expr.span);

                Expr::App(AppExpr {
                    lambda: Box::new(Expr::App(AppExpr {
                        lambda: Box::new(operator),
                        input:  Box::new(input),
                        span:   expr.span,
                    })),
                    input:  Box::new(Expr::Lam(LamExpr {
                        arg:    pat,
                        output: Box::new(output),
                        span:   expr.span,
                    })),
                    span:   expr.span,
                })
            }
        }
    }

    fn var_expr(&mut self, scope: &Scope, expr: ast::VarExpr) -> Expr {
        if let Some(value) = self.find_value(scope, expr.var.as_deref()) {
            return self.value_expr(value.clone(), expr.span);
        }

        let diagnostic = Diagnostic::error("undefined variable").label(expr.span, "found here");

        self.reporter.report(diagnostic);

        Expr::Error(ErrorExpr { span: expr.span })
    }

    fn value_expr(&mut self, value: ScopeValue, span: Span) -> Expr {
        match value.kind {
            ScopeValueKind::Import(..) => todo!(),

            ScopeValueKind::Def => {
                let def = self.defs.get(&value.var.unique()).unwrap();

                Expr::Def(DefExpr {
                    var: value.var,
                    ascribed: def.ty.is_some(),
                    span,
                })
            }

            ScopeValueKind::Local => Expr::Var(VarExpr {
                var: value.var,
                span,
            }),

            ScopeValueKind::Cons(type_) => Expr::Cons(ConsExpr {
                var: value.var,
                type_,
                span,
            }),

            ScopeValueKind::Method(trait_) => Expr::Method(MethodExpr {
                var: value.var,
                trait_,
                span,
            }),
        }
    }

    fn lam_expr(&mut self, scope: &Scope, expr: ast::LamExpr) -> Expr {
        self.expr_with_args(scope.clone(), expr.args, *expr.output)
    }

    fn app_expr(&mut self, scope: &Scope, expr: ast::AppExpr) -> Expr {
        let lambda = self.expr(scope, *expr.lambda);
        let input = self.expr(scope, *expr.input);

        Expr::App(AppExpr {
            lambda: Box::new(lambda),
            input:  Box::new(input),
            span:   expr.span,
        })
    }

    fn bin_expr(&mut self, scope: &Scope, expr: ast::BinExpr) -> Expr {
        let op = match self.find_value(scope, Some(&expr.op)) {
            Some(value) => self.value_expr(value.clone(), expr.span),

            None => {
                let diagnostic = Diagnostic::error("operator not defined").label(
                    expr.op.span(),
                    format!("`{}` used here", expr.op),
                );

                self.reporter.report(diagnostic);

                Expr::Error(ErrorExpr { span: expr.span })
            }
        };

        let lhs = self.expr(scope, *expr.lhs);
        let rhs = self.expr(scope, *expr.rhs);

        Expr::App(AppExpr {
            lambda: Box::new(Expr::App(AppExpr {
                lambda: Box::new(op),
                input:  Box::new(lhs),
                span:   expr.span,
            })),
            input:  Box::new(rhs),
            span:   expr.span,
        })
    }

    fn seq_expr(&mut self, scope: &Scope, expr: ast::SeqExpr) -> Expr {
        let input = self.expr(scope, *expr.input);
        let output = self.expr(scope, *expr.output);

        let Some(operator) = self.find_value(scope, Some(">>")) else {
            todo!()
        };

        let operator = self.value_expr(operator.clone(), expr.span);

        Expr::App(AppExpr {
            lambda: Box::new(Expr::App(AppExpr {
                lambda: Box::new(operator),
                input:  Box::new(input),
                span:   expr.span,
            })),
            input:  Box::new(output),
            span:   expr.span,
        })
    }

    fn match_expr(&mut self, scope: &Scope, expr: ast::MatchExpr) -> Expr {
        let input = self.expr(scope, *expr.input);

        let arms = expr
            .arms
            .into_iter()
            .map(|arm| {
                let mut scope = scope.clone();

                let pat = self.pat(&mut scope, arm.pat);
                let output = self.expr(&scope, arm.output);

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
            span: expr.span,
        })
    }

    fn error_expr(&mut self, _scope: &Scope, expr: ast::ErrorExpr) -> Expr {
        Expr::Error(ErrorExpr { span: expr.span })
    }
}
