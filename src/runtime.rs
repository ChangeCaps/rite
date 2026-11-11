use std::{cell::RefCell, collections::HashMap, fmt, mem, rc::Rc};

use crate::{
    ir::{Expr, Lit, Prim},
    primitive::{BinPrim, PrimNum},
    var::{Unique, Var},
};

#[derive(Default)]
pub struct Runtime {}

#[derive(Clone)]
pub enum Value<'a> {
    Str(String),
    Int(i64),
    Real(f64),
    Lam(Env<'a>, &'a Var, &'a Expr),
    Cons(&'a Var, Vec<Thunk<'a>>),
}

impl Value<'_> {
    pub fn as_i64(&self) -> i64 {
        match self {
            Self::Int(int) => *int,
            _ => panic!(),
        }
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            Self::Real(real) => *real,
            _ => panic!(),
        }
    }
}

impl fmt::Debug for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Str(str) => write!(f, "\"{str}\""),
            Self::Int(int) => write!(f, "{int}"),
            Self::Real(real) => write!(f, "{real}"),

            Self::Lam(..) => f.write_str("<lambda>"),

            Self::Cons(cons, args) => {
                if args.is_empty() {
                    return cons.fmt(f);
                }

                f.write_str("(")?;
                cons.fmt(f)?;

                for arg in args {
                    f.write_str(" ")?;
                    arg.fmt(f)?;
                }

                f.write_str(")")
            }
        }
    }
}

#[derive(Clone)]
pub struct Thunk<'a> {
    inner: Rc<RefCell<ThunkInner<'a>>>,
}

impl<'a> Thunk<'a> {
    pub fn new(env: Env<'a>, expr: &'a Expr) -> Self {
        Self {
            inner: Rc::new(RefCell::new(ThunkInner::Thunk(
                env, expr,
            ))),
        }
    }

    pub fn value(value: Value<'a>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(ThunkInner::Value(value))),
        }
    }

    pub fn force(&self, runtime: &mut Runtime) -> Value<'a> {
        let mut inner = self.inner.borrow_mut();

        match &mut *inner {
            ThunkInner::Value(value) => value.clone(),

            ThunkInner::Thunk(env, expr) => {
                let mut env = mem::take(env);
                let thunk = runtime.eval(&mut env, expr);
                let value = thunk.force(runtime);
                *inner = ThunkInner::Value(value.clone());
                value
            }
        }
    }

    pub fn set_env(&self, env: Env<'a>) {
        if let ThunkInner::Thunk(current, _) = &mut *self.inner.borrow_mut() {
            *current = env;
        }
    }
}

enum ThunkInner<'a> {
    Value(Value<'a>),
    Thunk(Env<'a>, &'a Expr),
}

impl fmt::Debug for Thunk<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self.inner.borrow() {
            ThunkInner::Value(value) => value.fmt(f),
            ThunkInner::Thunk(..) => f.write_str("<thunk>"),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Env<'a> {
    vars: HashMap<Unique, Thunk<'a>>,
}

impl<'a> Env<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn capture(&self, expr: &Expr) -> Self {
        fn recurse<'a>(env: &Env<'a>, cap: &mut Env<'a>, expr: &Expr) {
            match expr {
                Expr::Var(var) => {
                    if let Some(value) = env.vars.get(&var.unique()).cloned() {
                        cap.vars.insert(var.unique(), value);
                    }
                }

                Expr::Lit(..) => {}

                Expr::Lam(_, expr) => recurse(env, cap, expr),

                Expr::App(lam, input) => {
                    recurse(env, cap, lam);
                    recurse(env, cap, input);
                }

                Expr::Rec(vars, expr) => {
                    for (expr, _) in vars {
                        recurse(env, cap, expr);
                    }

                    recurse(env, cap, expr);
                }

                Expr::Let(input, _, output) => {
                    recurse(env, cap, input);
                    recurse(env, cap, output);
                }

                Expr::Cons(_, args) => {
                    for expr in args {
                        recurse(env, cap, expr);
                    }
                }

                Expr::Case(expr, alts, default) => {
                    recurse(env, cap, expr);

                    for alt in alts {
                        recurse(env, cap, &alt.expr);
                    }

                    if let Some(default) = default {
                        recurse(env, cap, default);
                    }
                }

                Expr::Prim(prim) => match prim {
                    Prim::Bin(_, lhs, rhs) => {
                        recurse(env, cap, lhs);
                        recurse(env, cap, rhs);
                    }
                },

                Expr::Note(_, expr) => recurse(env, cap, expr),
            }
        }

        let mut env = Env::new();
        recurse(self, &mut env, expr);
        env
    }
}

impl Runtime {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run<'a>(&mut self, expr: &'a Expr) -> Value<'a> {
        let mut env = Env::new();
        let thunk = self.eval(&mut env, expr);
        thunk.force(self)
    }

    fn eval<'a>(&mut self, env: &mut Env<'a>, expr: &'a Expr) -> Thunk<'a> {
        match expr {
            Expr::Var(var) => env
                .vars
                .get(&var.unique())
                .unwrap_or_else(|| panic!("{:?}", var))
                .clone(),

            Expr::Lit(lit) => match lit {
                Lit::Str(str) => Thunk::value(Value::Str(str.clone())),
                Lit::Int(int) => Thunk::value(Value::Int(*int)),
                Lit::Real(real) => Thunk::value(Value::Real(*real)),
            },

            Expr::Lam(var, expr) => {
                let env = env.capture(expr);
                Thunk::value(Value::Lam(env, var, expr))
            }

            Expr::App(lambda, input) => {
                let lambda = self.eval(env, lambda);
                let lambda = lambda.force(self);

                let input = self.eval(env, input);

                let Value::Lam(env, var, expr) = lambda else {
                    panic!();
                };

                let mut env = env.clone();
                env.vars.insert(var.unique(), input);
                self.eval(&mut env, expr)
            }

            Expr::Rec(vars, output) => {
                for (expr, var) in vars {
                    let value = Thunk::new(Env::new(), expr);
                    env.vars.insert(var.unique(), value);
                }

                for (expr, var) in vars {
                    let cap = env.capture(expr);
                    env.vars[&var.unique()].set_env(cap);
                }

                self.eval(env, output)
            }

            Expr::Let(input, var, output) => {
                let cap = env.capture(input);
                let input = Thunk::new(cap, input);
                env.vars.insert(var.unique(), input);
                self.eval(env, output)
            }

            Expr::Cons(var, args) => {
                let args = args.iter().map(|expr| self.eval(env, expr)).collect();
                Thunk::value(Value::Cons(var, args))
            }

            Expr::Case(input, alts, default) => {
                let input = self.eval(env, input);
                let input = input.force(self);

                let Value::Cons(var, args) = input else {
                    panic!();
                };

                for alt in alts {
                    if alt.var != *var {
                        continue;
                    };

                    for (var, arg) in alt.args.iter().zip(args) {
                        env.vars.insert(var.unique(), arg);
                    }

                    return self.eval(env, &alt.expr);
                }

                if let Some(default) = default {
                    return self.eval(env, default);
                }

                panic!();
            }

            Expr::Prim(prim) => match prim {
                Prim::Bin(prim, lhs, rhs) => {
                    let lhs = self.eval(env, lhs);
                    let rhs = self.eval(env, rhs);

                    let lhs = lhs.force(self);
                    let rhs = rhs.force(self);

                    Thunk::value(match prim {
                        BinPrim::Add(PrimNum::Int) => Value::Int(lhs.as_i64() + rhs.as_i64()),
                        BinPrim::Sub(PrimNum::Int) => Value::Int(lhs.as_i64() - rhs.as_i64()),
                        BinPrim::Mul(PrimNum::Int) => Value::Int(lhs.as_i64() * rhs.as_i64()),
                        BinPrim::Div(PrimNum::Int) => Value::Int(lhs.as_i64() / rhs.as_i64()),

                        _ => unimplemented!("{prim:?}"),
                    })
                }
            },

            Expr::Note(_, expr) => self.eval(env, expr),
        }
    }
}
