use std::{cell::RefCell, collections::HashMap, fmt, fs, mem, path::Path, rc::Rc};

use crate::{
    ir::{Case, Expr, Lit, Prim},
    primitive::{BinPrim, PrimNum, UnPrim},
    var::{Unique, Var},
};

#[derive(Default)]
pub struct Runtime {}

#[derive(Clone)]
pub enum Value<'a> {
    Io(Io<'a>),
    Str(String),
    Int(i64),
    Real(f64),
    Lam(Lam<'a>),
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

    pub fn as_str(&self) -> &str {
        match self {
            Self::Str(s) => s.as_str(),
            _ => panic!(),
        }
    }
}

impl PartialEq<Lit> for Value<'_> {
    fn eq(&self, other: &Lit) -> bool {
        match other {
            Lit::Str(str) => matches!(self, Value::Str(this) if this == str),
            Lit::Int(int) => matches!(self, Value::Int(this) if this == int),
            Lit::Real(real) => matches!(self, Value::Real(this) if this == real),
        }
    }
}

#[derive(Clone)]
pub enum Lam<'a> {
    Lam(Env<'a>, &'a Var, &'a Expr),
    Extern(ExternLam<'a>),
}

pub type ExternLam<'a> = Rc<dyn Fn(&mut Runtime, Thunk<'a>) -> Thunk<'a> + 'a>;

#[derive(Clone)]
pub enum Io<'a> {
    Pure(Thunk<'a>),
    Bind(Thunk<'a>, Thunk<'a>),
    Extern(ExternIo<'a>),
}

pub type ExternIo<'a> = Rc<dyn Fn(&mut Runtime) -> Value<'a> + 'a>;

impl fmt::Debug for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(..) => write!(f, "<io>"),

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

    pub fn set_env(&self, env: Env<'a>) {
        if let ThunkInner::Thunk(current, _) = &mut *self.inner.borrow_mut() {
            *current = env;
        }
    }

    pub fn lam(f: impl Fn(&mut Runtime, Thunk<'a>) -> Thunk<'a> + 'a) -> Self {
        Self::value(Value::Lam(Lam::Extern(Rc::new(f))))
    }

    pub fn io(f: impl Fn(&mut Runtime) -> Value<'a> + 'a) -> Self {
        Self::value(Value::Io(Io::Extern(Rc::new(f))))
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

                    match alts {
                        Case::Cons(cons) => {
                            for (_, _, expr) in cons {
                                recurse(env, cap, expr);
                            }
                        }

                        Case::Lit(lits) => {
                            for (_, expr) in lits {
                                recurse(env, cap, expr);
                            }
                        }
                    }

                    if let Some(default) = default {
                        recurse(env, cap, default);
                    }
                }

                Expr::Prim(prim) => match prim {
                    Prim::Extern(..) => {}

                    Prim::Un(_, input) => recurse(env, cap, input),

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
        let thunk = self.eval_io(thunk);
        self.force(thunk)
    }

    fn force<'a>(&mut self, thunk: Thunk<'a>) -> Value<'a> {
        let mut inner = thunk.inner.borrow_mut();

        match &mut *inner {
            ThunkInner::Value(value) => value.clone(),

            ThunkInner::Thunk(env, expr) => {
                let mut env = mem::take(env);
                let thunk = self.eval(&mut env, expr);
                let value = self.force(thunk);
                *inner = ThunkInner::Value(value.clone());
                value
            }
        }
    }

    fn eval_io<'a>(&mut self, thunk: Thunk<'a>) -> Thunk<'a> {
        let value = self.force(thunk);

        let Value::Io(io) = value else {
            panic!();
        };

        match io {
            Io::Pure(input) => input,

            Io::Bind(input, lambda) => {
                let input = self.eval_io(input);
                let lambda = self.force(lambda);

                let Value::Lam(lam) = lambda else {
                    panic!();
                };

                match lam {
                    Lam::Lam(mut env, var, output) => {
                        env.vars.insert(var.unique(), input);
                        let output = self.eval(&mut env, output);
                        self.eval_io(output)
                    }

                    Lam::Extern(f) => f(self, input),
                }
            }

            Io::Extern(f) => Thunk::value(f(self)),
        }
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
                Thunk::value(Value::Lam(Lam::Lam(env, var, expr)))
            }

            Expr::App(lambda, input) => {
                let lambda = self.eval(env, lambda);
                let lambda = self.force(lambda);

                let input = self.eval(env, input);

                let Value::Lam(lam) = lambda else {
                    panic!();
                };

                match lam {
                    Lam::Lam(mut env, var, output) => {
                        env.vars.insert(var.unique(), input);
                        self.eval(&mut env, output)
                    }

                    Lam::Extern(f) => f(self, input),
                }
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

            Expr::Cons(cons, args) => {
                let args = args.iter().map(|expr| self.eval(env, expr)).collect();
                Thunk::value(Value::Cons(cons, args))
            }

            Expr::Case(input, alts, default) => {
                let input = self.eval(env, input);
                let input = self.force(input);

                match alts {
                    Case::Cons(alts) => {
                        let Value::Cons(cons, args) = input else {
                            panic!("{input:?}");
                        };

                        for (cond, vars, expr) in alts {
                            if cond != cons {
                                continue;
                            };

                            for (var, arg) in vars.iter().zip(args) {
                                env.vars.insert(var.unique(), arg);
                            }

                            return self.eval(env, expr);
                        }
                    }

                    Case::Lit(alts) => {
                        for (cond, expr) in alts {
                            if input == *cond {
                                return self.eval(env, expr);
                            }
                        }
                    }
                }

                if let Some(default) = default {
                    return self.eval(env, default);
                }

                panic!();
            }

            Expr::Prim(prim) => match prim {
                Prim::Extern(name) => match name.as_str() {
                    "io::print" => Thunk::lam(|_, input| {
                        Thunk::io(move |rt| {
                            let input = rt.force(input.clone());
                            print!("{}", input.as_str());
                            Value::Int(0)
                        })
                    }),

                    "fs::readFile" => Thunk::lam(|_, input| {
                        Thunk::io(move |rt| {
                            let input = rt.force(input.clone());
                            let contents = fs::read_to_string(input.as_str()).unwrap_or_default();
                            Value::Str(contents)
                        })
                    }),

                    "fs::isFile" => Thunk::lam(|_, input| {
                        Thunk::io(move |rt| {
                            let input = rt.force(input.clone());
                            Value::Int(Path::new(input.as_str()).is_file() as i64)
                        })
                    }),

                    _ => panic!("invalid extern `{name}`"),
                },

                Prim::Un(prim, input) => {
                    let input = self.eval(env, input);

                    match prim {
                        UnPrim::IoPure => Thunk::value(Value::Io(Io::Pure(input))),

                        UnPrim::Str(PrimNum::Int) => {
                            let input = self.force(input);
                            Thunk::value(Value::Str(input.as_i64().to_string()))
                        }

                        _ => unimplemented!("{prim:?}"),
                    }
                }

                Prim::Bin(prim, lhs, rhs) => {
                    let lhs = self.eval(env, lhs);
                    let rhs = self.eval(env, rhs);

                    if *prim == BinPrim::IoBind {
                        return Thunk::value(Value::Io(Io::Bind(lhs, rhs)));
                    }

                    let lhs = self.force(lhs);
                    let rhs = self.force(rhs);

                    Thunk::value(match prim {
                        BinPrim::StrConcat => Value::Str(lhs.as_str().to_owned() + rhs.as_str()),

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
