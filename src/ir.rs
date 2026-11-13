use std::fmt;

use crate::{
    primitive::{BinPrim, UnPrim},
    var::Var,
};

#[derive(Clone)]
pub enum Expr {
    Var(Var),
    Lit(Lit),
    Lam(Var, Box<Expr>),
    App(Box<Expr>, Box<Expr>),
    Rec(Vec<(Expr, Var)>, Box<Expr>),
    Let(Box<Expr>, Var, Box<Expr>),
    Cons(Cons, Vec<Expr>),
    Case(Box<Expr>, Vec<Alt>, Option<Box<Expr>>),
    Prim(Prim),
    Note(String, Box<Expr>),
}

#[derive(Clone)]
pub enum Prim {
    Un(UnPrim, Box<Expr>),
    Bin(BinPrim, Box<Expr>, Box<Expr>),
}

#[derive(Clone)]
pub enum Lit {
    Str(String),
    Int(i64),
    Real(f64),
}

#[derive(Clone)]
pub struct Alt {
    pub cons: Cons,
    pub args: Vec<Var>,
    pub expr: Expr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cons {
    Var(Var),
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn fmt_expr(expr: &Expr, i: usize, p: u8) -> String {
            fn fmt_paren(input: String, p: u8, q: u8) -> String {
                match p > q {
                    true => format!("({input})"),
                    false => input,
                }
            }

            fn fmt_line(i: usize) -> String {
                format!("\n{}", "  ".repeat(i))
            }

            fn fmt_var(var: &Var) -> String {
                var.as_str()
                    .map(Into::into)
                    .unwrap_or_else(|| format!("_{}", var.unique()))
            }

            fn fmt_cons(cons: &Cons) -> String {
                match cons {
                    Cons::Var(var) => fmt_var(var),
                }
            }

            match expr {
                Expr::Var(var) => fmt_var(var),

                Expr::Lit(lit) => match lit {
                    Lit::Str(str) => format!("\"{str}\""),
                    Lit::Int(int) => format!("{int}_int"),
                    Lit::Real(real) => format!("{real}_real"),
                },

                Expr::Lam(var, expr) => {
                    let expr = fmt_expr(expr, i + 1, p);

                    format!(
                        "{}{}",
                        fmt_line(i),
                        fmt_paren(
                            format!("\\{}. {}", fmt_var(var), expr),
                            p,
                            0,
                        ),
                    )
                }

                Expr::App(lambda, input) => {
                    let lambda = fmt_expr(lambda, i, 1);
                    let input = fmt_expr(input, i, 1);

                    fmt_paren(format!("{lambda} {input}"), p, 0)
                }

                Expr::Rec(vars, expr) => {
                    let line = fmt_line(i);

                    let vars = vars
                        .iter()
                        .map(|(expr, var)| {
                            let expr = fmt_expr(expr, i + 2, 0);
                            let var = fmt_var(var);

                            format!("{line}  {var} = {expr}")
                        })
                        .collect::<Vec<_>>();

                    let expr = fmt_expr(expr, i + 1, 0);

                    format!(
                        "letrec{}{line}in {}",
                        vars.join(&line),
                        expr,
                    )
                }

                Expr::Let(input, var, output) => fmt_paren(
                    format!(
                        "{}let {} = {}{}in {}",
                        fmt_line(i),
                        fmt_var(var),
                        fmt_expr(input, i + 1, 0),
                        fmt_line(i),
                        fmt_expr(output, i + 1, 0),
                    ),
                    p,
                    0,
                ),

                Expr::Cons(cons, args) => {
                    let mut args = args
                        .iter()
                        .map(|arg| fmt_expr(arg, i, 1))
                        .collect::<Vec<_>>();

                    args.insert(0, fmt_cons(cons));

                    args.join(" ")
                }

                Expr::Case(input, alts, default) => {
                    let mut alts = alts
                        .iter()
                        .map(|alt| {
                            let mut args = alt.args.iter().map(fmt_var).collect::<Vec<_>>();
                            args.insert(0, fmt_cons(&alt.cons));

                            format!(
                                "{} -> {}",
                                args.join(" "),
                                fmt_expr(&alt.expr, i + 2, 0)
                            )
                        })
                        .collect::<Vec<_>>();

                    if let Some(default) = default {
                        alts.push(format!(
                            "_ -> {}",
                            fmt_expr(default, i + 2, 0)
                        ));
                    }

                    fmt_paren(
                        format!(
                            "{}case {}{}{}",
                            fmt_line(i),
                            fmt_expr(input, i, 0),
                            fmt_line(i + 1),
                            alts.join(&fmt_line(i + 1)),
                        ),
                        p,
                        0,
                    )
                }

                Expr::Prim(prim) => match prim {
                    Prim::Un(prim, input) => {
                        let input = fmt_expr(input, i, 0);

                        format!("({prim:?} {input})")
                    }

                    Prim::Bin(prim, lhs, rhs) => {
                        let lhs = fmt_expr(lhs, i, 0);
                        let rhs = fmt_expr(rhs, i, 0);

                        format!("({prim:?} {lhs} {rhs})")
                    }
                },

                Expr::Note(note, expr) => {
                    format!(
                        "# {note}{}{}",
                        fmt_line(i),
                        fmt_expr(expr, i, p),
                    )
                }
            }
        }

        f.write_str(&fmt_expr(self, 0, 0))
    }
}
