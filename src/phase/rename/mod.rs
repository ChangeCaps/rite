use crate::{
    ast::{
        AppExpr, AppTy, AscribeStmt, AssignStmt, ErrorExpr, Expr, LamExpr, LamTy, LetExpr, Parse,
        Rename, Stmt, Ty, Unique, Var, VarExpr, VarTy,
    },
    diagnostic::{Diagnostic, Reporter},
};

pub fn rename(reporter: &dyn Reporter, stmts: Vec<Stmt<Parse>>) -> Vec<Stmt<Rename>> {
    let mut state = State::new(reporter);
    state.stmts(stmts)
}

struct State<'a> {
    reporter: &'a dyn Reporter,
    vars:     Vec<Var>,
    tys:      Vec<Var>,
}

impl<'a> State<'a> {
    fn new(reporter: &'a dyn Reporter) -> Self {
        Self {
            reporter,
            vars: Vec::new(),
            tys: Vec::new(),
        }
    }

    fn scoped<T>(&mut self, f: impl FnOnce(&mut State<'_>) -> T) -> T {
        let vars_len = self.vars.len();
        let tys_len = self.tys.len();

        let output = f(self);

        self.vars.truncate(vars_len);
        self.tys.truncate(tys_len);

        output
    }

    fn find_var(&self, name: &str) -> Option<&Var> {
        self.vars.iter().rev().find(|var| var.ident == name)
    }

    fn find_ty(&self, name: &str) -> Option<&Var> {
        self.tys.iter().rev().find(|var| var.ident == name)
    }

    fn stmts(&mut self, stmts: Vec<Stmt<Parse>>) -> Vec<Stmt<Rename>> {
        stmts.into_iter().map(|stmt| self.stmt(stmt)).collect()
    }

    fn stmt(&mut self, stmt: Stmt<Parse>) -> Stmt<Rename> {
        match stmt {
            Stmt::Ascribe(stmt) => self.ascribe_stmt(stmt),
            Stmt::Assign(stmt) => self.assign_stmt(stmt),
            Stmt::Error(stmt) => Stmt::Error(stmt),
        }
    }

    fn ascribe_stmt(&mut self, _stmt: AscribeStmt<Parse>) -> Stmt<Rename> {
        todo!()
    }

    fn assign_stmt(&mut self, stmt: AssignStmt<Parse>) -> Stmt<Rename> {
        let var = Var {
            ident:  stmt.var,
            unique: Unique::new(),
            span:   stmt.span,
        };

        self.vars.push(var.clone());

        let expr = self.expr(stmt.expr);

        Stmt::Assign(AssignStmt {
            var,
            expr,
            span: stmt.span,
        })
    }

    fn expr(&mut self, expr: Expr<Parse>) -> Expr<Rename> {
        match expr {
            Expr::Let(expr) => self.let_expr(expr),
            Expr::Var(expr) => self.var_expr(expr),
            Expr::Lam(expr) => self.lam_expr(expr),
            Expr::App(expr) => self.app_expr(expr),
            Expr::Error(expr) => self.error_expr(expr),
        }
    }

    fn let_expr(&mut self, expr: LetExpr<Parse>) -> Expr<Rename> {
        self.scoped(|state| {
            let input = Box::new(state.expr(*expr.input));

            let name = Var {
                ident:  expr.var,
                unique: Unique::new(),
                span:   expr.span,
            };

            state.vars.push(name.clone());

            let output = Box::new(state.expr(*expr.output));

            Expr::Let(LetExpr {
                var: name,
                input,
                output,
                ty: expr.ty,
                span: expr.span,
            })
        })
    }

    fn var_expr(&mut self, expr: VarExpr<Parse>) -> Expr<Rename> {
        match self.find_var(&expr.var) {
            Some(var) => Expr::Var(VarExpr {
                var:  var.clone(),
                ty:   expr.ty,
                span: expr.span,
            }),

            None => {
                let diagnostic = Diagnostic::error(format!(
                    "undefined variable `{}`",
                    expr.var,
                ))
                .label(expr.span, "found here");

                self.reporter.report(diagnostic);

                Expr::Error(ErrorExpr {
                    ty:   expr.ty,
                    span: expr.span,
                })
            }
        }
    }

    fn lam_expr(&mut self, expr: LamExpr<Parse>) -> Expr<Rename> {
        self.scoped(|state| {
            let input = Var {
                ident:  expr.var,
                unique: Unique::new(),
                span:   expr.span,
            };

            state.vars.push(input.clone());

            let output = Box::new(state.expr(*expr.output));

            Expr::Lam(LamExpr {
                var: input,
                output,
                ty: expr.ty,
                span: expr.span,
            })
        })
    }

    fn app_expr(&mut self, expr: AppExpr<Parse>) -> Expr<Rename> {
        let lambda = Box::new(self.expr(*expr.lambda));
        let input = Box::new(self.expr(*expr.input));

        Expr::App(AppExpr {
            lambda,
            input,
            ty: expr.ty,
            span: expr.span,
        })
    }

    fn error_expr(&mut self, expr: ErrorExpr<Parse>) -> Expr<Rename> {
        Expr::Error(ErrorExpr {
            ty:   expr.ty,
            span: expr.span,
        })
    }

    fn ty(&mut self, ty: Ty<Parse>) -> Ty<Rename> {
        match ty {
            Ty::Ty(ty) => Ty::Ty(ty),
            Ty::Var(ty) => self.var_ty(ty),
            Ty::Lam(ty) => self.lam_ty(ty),
            Ty::App(ty) => self.app_ty(ty),
            Ty::Error(ty) => Ty::Error(ty),
        }
    }

    fn var_ty(&mut self, ty: VarTy<Parse>) -> Ty<Rename> {
        match self.find_ty(&ty.var) {
            Some(var) => Ty::Var(VarTy {
                var:  var.clone(),
                span: ty.span,
            }),

            None => {
                let var = Var {
                    ident:  ty.var,
                    unique: Unique::new(),
                    span:   ty.span,
                };

                self.tys.push(var.clone());

                Ty::Var(VarTy { var, span: ty.span })
            }
        }
    }

    fn lam_ty(&mut self, ty: LamTy<Parse>) -> Ty<Rename> {
        let input = self.ty(*ty.input);
        let output = self.ty(*ty.output);

        Ty::Lam(LamTy {
            input:  Box::new(input),
            output: Box::new(output),
            span:   ty.span,
        })
    }

    fn app_ty(&mut self, ty: AppTy<Parse>) -> Ty<Rename> {
        let lambda = self.ty(*ty.lambda);
        let input = self.ty(*ty.input);

        Ty::App(AppTy {
            lambda: Box::new(lambda),
            input:  Box::new(input),
            span:   ty.span,
        })
    }
}
