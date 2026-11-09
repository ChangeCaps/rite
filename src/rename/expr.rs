use crate::{
    ast,
    diagnostic::{Diagnostic, Span},
    rename::{Pat, State, ValueKind},
    var::Var,
};

pub enum Expr {
    Let(LetExpr),
    Var(VarExpr),
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
    pub fn expr(&mut self, expr: ast::Expr) -> Expr {
        match expr {
            ast::Expr::Let(expr) => self.let_expr(expr),
            ast::Expr::Var(expr) => self.var_expr(expr),
            ast::Expr::Lam(expr) => self.lam_expr(expr),
            ast::Expr::App(expr) => self.app_expr(expr),
            ast::Expr::Bin(expr) => self.bin_expr(expr),
            ast::Expr::Match(expr) => self.match_expr(expr),
            ast::Expr::Error(expr) => self.error_expr(expr),
        }
    }

    fn let_expr(&mut self, expr: ast::LetExpr) -> Expr {
        self.value_scope(|state| {
            let input = Box::new(state.expr(*expr.input));

            let name = Var::from(expr.var);

            state.values.push((name.clone(), ValueKind::Local));

            let output = Box::new(state.expr(*expr.output));

            Expr::Let(LetExpr {
                var: name,
                input,
                output,
                span: expr.span,
            })
        })
    }

    fn var_expr(&mut self, expr: ast::VarExpr) -> Expr {
        if let Some((var, kind)) = self.find_value(expr.var.as_ref()) {
            return self.var_to_expr(var.clone(), kind.clone(), expr.span);
        }

        let diagnostic = Diagnostic::error("undefined variable").label(expr.span, "found here");

        self.reporter.report(diagnostic);

        Expr::Error(ErrorExpr { span: expr.span })
    }

    fn var_to_expr(&mut self, var: Var, kind: ValueKind, span: Span) -> Expr {
        match kind {
            ValueKind::Local | ValueKind::Def => Expr::Var(VarExpr { var, span }),
            ValueKind::Cons(type_) => Expr::Cons(ConsExpr { var, type_, span }),
            ValueKind::Method(trait_) => Expr::Method(MethodExpr { var, trait_, span }),
        }
    }

    fn lam_expr(&mut self, expr: ast::LamExpr) -> Expr {
        self.value_scope(|state| {
            let input = Var::from(expr.var);

            state.values.push((input.clone(), ValueKind::Local));

            let output = Box::new(state.expr(*expr.output));

            Expr::Lam(LamExpr {
                var: input,
                output,
                span: expr.span,
            })
        })
    }

    fn app_expr(&mut self, expr: ast::AppExpr) -> Expr {
        let lambda = Box::new(self.expr(*expr.lambda));
        let input = Box::new(self.expr(*expr.input));

        Expr::App(AppExpr {
            lambda,
            input,
            span: expr.span,
        })
    }

    fn bin_expr(&mut self, expr: ast::BinExpr) -> Expr {
        let op = match self.find_value(Some(&expr.op)) {
            Some((var, kind)) => self.var_to_expr(var.clone(), kind.clone(), expr.span),
            None => {
                let diagnostic = Diagnostic::error("operator not defined").label(
                    expr.op.span(),
                    format!("`{}` used here", expr.op),
                );

                self.reporter.report(diagnostic);

                Expr::Error(ErrorExpr { span: expr.span })
            }
        };

        let lhs = self.expr(*expr.lhs);
        let rhs = self.expr(*expr.rhs);

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

    fn match_expr(&mut self, expr: ast::MatchExpr) -> Expr {
        let input = self.expr(*expr.input);

        let arms = expr
            .arms
            .into_iter()
            .map(|arm| {
                self.value_scope(|state| {
                    let pat = state.pat(arm.pat);
                    let output = state.expr(arm.output);

                    MatchArm {
                        pat,
                        output,
                        span: arm.span,
                    }
                })
            })
            .collect();

        Expr::Match(MatchExpr {
            input: Box::new(input),
            arms,
            span: expr.span,
        })
    }

    fn error_expr(&mut self, expr: ast::ErrorExpr) -> Expr {
        Expr::Error(ErrorExpr { span: expr.span })
    }
}
