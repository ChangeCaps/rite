use std::collections::HashMap;

use crate::{
    ir::{Alt, Expr},
    typecheck::{self, Impl, Module, Pat, Ty, TypeCons},
    var::{Unique, Var},
};

pub fn lower(module: Module) -> Expr {
    let mut state = State::new(&module);

    let main = module
        .defs
        .values()
        .find(|def| def.var.as_str() == Some("main"))
        .unwrap();

    let main = state.expr(&main.expr);

    Expr::Rec(state.globals, Box::new(main))
}

struct State<'a> {
    module:  &'a Module,
    subst:   HashMap<Unique, Ty>,
    defs:    HashMap<(Unique, Ty), Var>,
    globals: Vec<(Expr, Var)>,
}

impl<'a> State<'a> {
    fn new(module: &'a Module) -> Self {
        Self {
            module,
            subst: HashMap::new(),
            defs: HashMap::new(),
            globals: Vec::new(),
        }
    }

    fn format_ty(&self, ty: &Ty) -> String {
        self.module.tycx.format_ty(&self.module.kinds, ty)
    }

    fn find_impl(&self, trait_: &Var, ty: &Ty) -> Option<&'a Impl> {
        self.module.tycx.find_impl(&self.module.kinds, trait_, ty)
    }

    fn unify(&mut self, lhs: &Ty, rhs: &Ty) {
        if let Some(lhs) = self.subst_shallow(lhs).cloned() {
            return self.unify(&lhs, rhs);
        } else if let Some(rhs) = self.subst_shallow(rhs).cloned() {
            return self.unify(lhs, &rhs);
        }

        match (lhs, rhs) {
            (Ty::Var(var), ty) => {
                self.subst.insert(var.unique(), ty.clone());
            }

            (_, Ty::Var(_)) => unreachable!(),

            (Ty::Cons(lhs_cons, lhs_args), Ty::Cons(rhs_cons, rhs_args)) => {
                debug_assert_eq!(lhs_cons, rhs_cons);

                for (lhs, rhs) in lhs_args.iter().zip(rhs_args) {
                    self.unify(lhs, rhs);
                }
            }
        }
    }

    fn subst_shallow(&self, ty: &Ty) -> Option<&Ty> {
        match ty {
            Ty::Var(var) => self.subst.get(&var.unique()).or_else(|| {
                self.module
                    .kinds
                    .get(&var.unique())
                    .and_then(|kind| kind.subst.as_ref())
            }),

            Ty::Cons(..) => None,
        }
    }

    fn subst(&self, ty: Ty) -> Ty {
        if let Some(ty) = self.subst_shallow(&ty) {
            return self.subst(ty.clone());
        }

        match ty {
            Ty::Var(_) => ty,

            Ty::Cons(cons, args) => {
                let args = args.into_iter().map(|ty| self.subst(ty)).collect();
                Ty::Cons(cons, args)
            }
        }
    }

    fn get_cons(&self, type_: &Var, var: &Var) -> &TypeCons {
        self.module.tycx.get_cons(type_, var)
    }

    fn expr(&mut self, expr: &typecheck::Expr) -> Expr {
        match expr {
            typecheck::Expr::Let(expr) => self.let_expr(expr),
            typecheck::Expr::Var(expr) => self.var_expr(expr),
            typecheck::Expr::Def(expr) => self.def_expr(expr),
            typecheck::Expr::Cons(expr) => self.cons_expr(expr),
            typecheck::Expr::Method(expr) => self.method_expr(expr),
            typecheck::Expr::Lam(expr) => self.lam_expr(expr),
            typecheck::Expr::App(expr) => self.app_expr(expr),
            typecheck::Expr::Match(expr) => self.match_expr(expr),
            typecheck::Expr::Error(expr) => self.error_expr(expr),
        }
    }

    fn let_expr(&mut self, expr: &typecheck::LetExpr) -> Expr {
        let input = self.expr(&expr.input);
        let output = self.expr(&expr.output);

        Expr::Let(
            Box::new(input),
            expr.var.clone(),
            Box::new(output),
        )
    }

    fn var_expr(&mut self, expr: &typecheck::VarExpr) -> Expr {
        Expr::Var(expr.var.clone())
    }

    fn def_expr(&mut self, expr: &typecheck::DefExpr) -> Expr {
        let ty = self.subst(expr.ty.clone());
        let key = (expr.var.unique(), ty.clone());

        if let Some(var) = self.defs.get(&key) {
            return Expr::Var(var.clone());
        }

        let def = self.module.defs.get(&expr.var.unique()).unwrap();

        let var = Var::from(def.var.ident().cloned());
        self.defs.insert(key, var.clone());

        let subst = self.subst.clone();
        self.unify(&Ty::Var(def.var.clone()), &ty);

        let expr = self.expr(&def.expr);
        self.subst = subst;

        let ty = self.format_ty(&ty);
        let expr = Expr::Note(ty, Box::new(expr));

        self.globals.push((expr, var.clone()));

        Expr::Var(var)
    }

    fn cons_expr(&mut self, expr: &typecheck::ConsExpr) -> Expr {
        let cons = self.get_cons(&expr.type_, &expr.var);
        let args = cons.args.iter().map(|_| Var::unnamed()).collect::<Vec<_>>();

        let cons = Expr::Cons(
            expr.var.clone(),
            args.iter().cloned().map(Expr::Var).collect(),
        );

        args.into_iter().rfold(cons, |expr, arg| {
            Expr::Lam(arg, Box::new(expr))
        })
    }

    fn method_expr(&mut self, expr: &typecheck::MethodExpr) -> Expr {
        let ty = self.subst(expr.ty.clone());
        let key = (expr.var.unique(), ty.clone());

        if let Some(var) = self.defs.get(&key) {
            return Expr::Var(var.clone());
        }

        let var = Var::from(expr.var.ident().cloned());
        self.defs.insert(key, var.clone());

        let impl_ = self.find_impl(&expr.trait_, &expr.input).unwrap();
        let method = impl_
            .methods
            .iter()
            .find(|method| method.var == expr.var)
            .unwrap();

        let subst = self.subst.clone();
        self.unify(&Ty::Var(method.var.clone()), &ty);

        let expr = self.expr(&method.expr);
        self.subst = subst;

        let ty = self.format_ty(&ty);
        let expr = Expr::Note(ty, Box::new(expr));

        self.globals.push((expr, var.clone()));

        Expr::Var(var)
    }

    fn lam_expr(&mut self, expr: &typecheck::LamExpr) -> Expr {
        let output = self.expr(&expr.output);

        Expr::Lam(expr.var.clone(), Box::new(output))
    }

    fn app_expr(&mut self, expr: &typecheck::AppExpr) -> Expr {
        let lambda = self.expr(&expr.lambda);
        let input = self.expr(&expr.input);

        Expr::App(Box::new(lambda), Box::new(input))
    }

    fn match_expr(&mut self, expr: &typecheck::MatchExpr) -> Expr {
        let input = self.expr(&expr.input);

        let rows = expr
            .arms
            .iter()
            .map(|arm| {
                let output = self.expr(&arm.output);

                Row {
                    pats: vec![&arm.pat],
                    output,
                }
            })
            .collect();

        self.bind(vec![input], rows)
    }

    fn error_expr(&mut self, _expr: &typecheck::ErrorExpr) -> Expr {
        unreachable!()
    }

    fn bind(&mut self, mut inputs: Vec<Expr>, mut rows: Vec<Row<'_>>) -> Expr {
        let Some(input) = inputs.pop() else {
            let arm = rows.pop().unwrap();
            return arm.output;
        };

        let pat = rows.first().unwrap().pats.first().unwrap();

        match pat {
            Pat::Bind(..) => self.bind_bind(input, inputs, rows),
            Pat::Cons(..) => self.bind_cons(input, inputs, rows),
            Pat::Error(..) => unreachable!(),
        }
    }

    fn bind_bind(&mut self, input: Expr, inputs: Vec<Expr>, mut rows: Vec<Row<'_>>) -> Expr {
        for arm in &mut rows {
            let pat = arm.pats.pop().unwrap();

            let Pat::Bind(pat) = pat else {
                panic!();
            };

            arm.output = match pat.var {
                Some(ref var) => Expr::Let(
                    Box::new(input.clone()),
                    var.clone(),
                    Box::new(arm.output.clone()),
                ),
                None => arm.output.clone(),
            };
        }

        self.bind(inputs, rows)
    }

    fn bind_cons(&mut self, input: Expr, inputs: Vec<Expr>, rows: Vec<Row<'_>>) -> Expr {
        let mut conss = Vec::new();
        let mut defaults = Vec::new();

        for mut arm in rows {
            let pat = arm.pats.pop().unwrap();

            match pat {
                Pat::Bind(_) => {
                    defaults.push(arm);
                }

                Pat::Cons(pat) => {
                    arm.pats.extend(pat.args.iter());

                    let args: Vec<_> = pat.args.iter().map(|_| Var::unnamed()).collect();

                    conss.push((pat.var.clone(), args, vec![arm]));
                }

                _ => panic!(),
            }
        }

        let alts = conss
            .into_iter()
            .map(|(var, args, rows)| {
                let mut inputs = inputs.clone();
                inputs.extend(args.iter().cloned().map(Expr::Var));

                let expr = self.bind(inputs.clone(), rows);

                Alt { var, args, expr }
            })
            .collect();

        let default = match defaults.is_empty() {
            true => None,
            false => {
                let expr = self.bind(inputs, defaults);
                Some(Box::new(expr))
            }
        };

        Expr::Case(Box::new(input), alts, default)
    }
}

struct Row<'a> {
    pats:   Vec<&'a Pat>,
    output: Expr,
}
