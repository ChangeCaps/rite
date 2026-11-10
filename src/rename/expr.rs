use crate::{
    ast,
    diagnostic::{Diagnostic, Span},
    rename::{Pat, Scope, State, Value, ValueKind, Vis},
    var::Var,
};

pub enum Expr {
    Let(LetExpr),
    Var(VarExpr),
    Def(DefExpr),
    Cons(ConsExpr),
    Method(MethodExpr),
    Lam(LamExpr),
    App(AppExpr),
    Match(MatchExpr),
    Error(ErrorExpr),
}

pub struct LetExpr {
    pub var:    Var,
    pub input:  Box<Expr>,
    pub output: Box<Expr>,
    pub span:   Span,
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

pub struct LamExpr {
    pub var:    Var,
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

pub struct ErrorExpr {
    pub span: Span,
}

impl State<'_> {
    pub fn expr(&mut self, scope: &Scope, expr: ast::Expr) -> Expr {
        match expr {
            ast::Expr::Lit(expr) => self.lit_expr(scope, expr),
            ast::Expr::Let(expr) => self.let_expr(scope, expr),
            ast::Expr::Var(expr) => self.var_expr(scope, expr),
            ast::Expr::Lam(expr) => self.lam_expr(scope, expr),
            ast::Expr::App(expr) => self.app_expr(scope, expr),
            ast::Expr::Bin(expr) => self.bin_expr(scope, expr),
            ast::Expr::Match(expr) => self.match_expr(scope, expr),
            ast::Expr::Error(expr) => self.error_expr(scope, expr),
        }
    }

    fn lit_expr(&mut self, _scope: &Scope, _expr: ast::LitExpr) -> Expr {
        /*
        let lit = match expr.lit {
            ast::Lit::Str(_) => todo!(),
            ast::Lit::Int(_) => todo!(),
            ast::Lit::Real(_) => todo!(),
        };
        */

        todo!()
    }

    fn let_expr(&mut self, scope: &Scope, expr: ast::LetExpr) -> Expr {
        let input = self.expr(scope, *expr.input);

        let mut scope = scope.clone();
        let var = Var::from(expr.var);

        let value = Value {
            vis:  Vis::Local,
            var:  var.clone(),
            kind: ValueKind::Local,
        };

        scope.values.push(value);

        let output = self.expr(&scope, *expr.output);

        Expr::Let(LetExpr {
            var,
            input: Box::new(input),
            output: Box::new(output),
            span: expr.span,
        })
    }

    fn var_expr(&mut self, scope: &Scope, expr: ast::VarExpr) -> Expr {
        if let Some(value) = self.find_value(scope, expr.var.as_ref()) {
            return self.var_to_expr(
                value.var.clone(),
                value.kind.clone(),
                expr.span,
            );
        }

        let diagnostic = Diagnostic::error("undefined variable").label(expr.span, "found here");

        self.reporter.report(diagnostic);

        Expr::Error(ErrorExpr { span: expr.span })
    }

    fn var_to_expr(&mut self, var: Var, kind: ValueKind, span: Span) -> Expr {
        match kind {
            ValueKind::Import(..) => todo!(),

            ValueKind::Def => {
                let def = self.defs.get(&var.unique()).unwrap();

                Expr::Def(DefExpr {
                    var,
                    ascribed: def.ty.is_some(),
                    span,
                })
            }

            ValueKind::Local => Expr::Var(VarExpr { var, span }),
            ValueKind::Cons(type_) => Expr::Cons(ConsExpr { var, type_, span }),
            ValueKind::Method(trait_) => Expr::Method(MethodExpr { var, trait_, span }),
        }
    }

    fn lam_expr(&mut self, scope: &Scope, expr: ast::LamExpr) -> Expr {
        let var = Var::from(expr.var);

        let value = Value {
            vis:  Vis::Local,
            var:  var.clone(),
            kind: ValueKind::Local,
        };

        let mut scope = scope.clone();
        scope.values.push(value);

        let output = self.expr(&scope, *expr.output);

        Expr::Lam(LamExpr {
            var,
            output: Box::new(output),
            span: expr.span,
        })
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
            Some(value) => self.var_to_expr(
                value.var.clone(),
                value.kind.clone(),
                expr.span,
            ),

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
