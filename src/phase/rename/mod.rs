use crate::{
    ast::{
        AppExpr, AppTy, AscribeStmt, AssignStmt, Assignment, ErrorExpr, Expr, LamExpr, LamTy,
        LetExpr, Parse, Rename, RenamedModule, Stmt, TypeLit, Unique, Var, VarExpr, VarTy,
    },
    diagnostic::{Diagnostic, Reporter},
};

pub fn rename(reporter: &dyn Reporter, stmts: Vec<Stmt<Parse>>) -> RenamedModule {
    let mut state = State::new(reporter);
    state.stmts(stmts);

    RenamedModule {
        assignments: state.assignments,
    }
}

struct State<'a> {
    reporter:    &'a dyn Reporter,
    vars:        Vec<Var>,
    tys:         Vec<Var>,
    assignments: Vec<Assignment<Rename>>,
}

impl<'a> State<'a> {
    fn new(reporter: &'a dyn Reporter) -> Self {
        Self {
            reporter,
            vars: Vec::new(),
            tys: Vec::new(),
            assignments: Vec::new(),
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

    fn stmts(&mut self, stmts: Vec<Stmt<Parse>>) {
        let (assign, ascribe) = stmts.into_iter().fold(
            (Vec::new(), Vec::new()),
            |(mut assign, mut ascribe), stmt| {
                match stmt {
                    Stmt::Assign(stmt) => assign.push(stmt),
                    Stmt::Ascribe(stmt) => ascribe.push(stmt),
                    Stmt::Error(..) => {}
                }

                (assign, ascribe)
            },
        );

        for stmt in assign {
            self.assign_stmt(stmt);
        }

        for stmt in ascribe {
            self.ascribe_stmt(stmt);
        }
    }

    fn ascribe_stmt(&mut self, stmt: AscribeStmt<Parse>) {
        let ty = self.ty(stmt.ty);

        let Some(var) = self.find_var(&stmt.var) else {
            let diagnostic = Diagnostic::error("undefined variable").label(stmt.span, "found here");
            self.reporter.report(diagnostic);
            return;
        };

        let unique = var.unique;
        let assignment = self
            .assignments
            .iter_mut()
            .find(|assignment| assignment.var.unique == unique)
            .unwrap();

        assignment.ty = Some(ty);
    }

    fn assign_stmt(&mut self, stmt: AssignStmt<Parse>) {
        let var = Var {
            ident:  stmt.var,
            unique: Unique::new(),
            span:   stmt.span,
        };

        self.vars.push(var.clone());

        let expr = self.expr(stmt.expr);

        self.assignments.push(Assignment {
            ty: None,
            var,
            expr,
            span: stmt.span,
        });
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

    fn ty(&mut self, ty: TypeLit<Parse>) -> TypeLit<Rename> {
        match ty {
            TypeLit::Ty(ty) => TypeLit::Ty(ty),
            TypeLit::Var(ty) => self.var_ty(ty),
            TypeLit::Lam(ty) => self.lam_ty(ty),
            TypeLit::App(ty) => self.app_ty(ty),
            TypeLit::Error(ty) => TypeLit::Error(ty),
        }
    }

    fn var_ty(&mut self, ty: VarTy<Parse>) -> TypeLit<Rename> {
        match self.find_ty(&ty.var) {
            Some(var) => TypeLit::Var(VarTy {
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

                TypeLit::Var(VarTy { var, span: ty.span })
            }
        }
    }

    fn lam_ty(&mut self, ty: LamTy<Parse>) -> TypeLit<Rename> {
        let input = self.ty(*ty.input);
        let output = self.ty(*ty.output);

        TypeLit::Lam(LamTy {
            input:  Box::new(input),
            output: Box::new(output),
            span:   ty.span,
        })
    }

    fn app_ty(&mut self, ty: AppTy<Parse>) -> TypeLit<Rename> {
        let lambda = self.ty(*ty.lambda);
        let input = self.ty(*ty.input);

        TypeLit::App(AppTy {
            lambda: Box::new(lambda),
            input:  Box::new(input),
            span:   ty.span,
        })
    }
}
