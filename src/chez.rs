use crate::{
    ir::{Case, Expr, Lit, Prim},
    primitive::{BinPrim, PrimNum, UnPrim},
    var::Var,
};

pub fn codegen(expr: &Expr) -> String {
    include_str!("chez-prelude.ss").to_string() + &gen_expr(expr)
}

fn gen_var(var: &Var) -> String {
    if let Some(name) = var.as_str()
        && name.chars().all(|c| c.is_ascii_alphanumeric())
    {
        format!("{name}_{}", var.unique().data())
    } else {
        format!("_{}", var.unique().data())
    }
}

fn gen_cons(var: &Var) -> String {
    format!("'{}", gen_var(var))
}

fn gen_lit(lit: &Lit) -> String {
    match lit {
        Lit::Str(x) => {
            let escaped = x
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
                .replace('\t', "\\t")
                .replace('\r', "\\r");

            format!("\"{escaped}\"")
        }

        Lit::Int(x) => format!("{x}"),
        Lit::Real(x) => format!("{x}"),
    }
}

fn gen_expr(expr: &Expr) -> String {
    match expr {
        Expr::Var(var) => gen_var(var),
        Expr::Lit(lit) => gen_lit(lit),

        Expr::Lam(var, expr) => {
            format!(
                "(lambda ({}) {})",
                gen_var(var),
                gen_expr(expr),
            )
        }

        Expr::App(lambda, input) => format!(
            "(apply-lazy {} {})",
            gen_expr(lambda),
            gen_expr(input),
        ),

        Expr::Rec(vars, expr) => {
            let vars = vars
                .iter()
                .map(|(expr, var)| {
                    format!(
                        "(define {} (thunk (lambda () {})))",
                        gen_var(var),
                        gen_expr(expr)
                    )
                })
                .collect::<Vec<_>>()
                .join("\n\n");

            format!("{vars}\n\n({})", gen_expr(expr))
        }

        Expr::Let(input, var, output) => {
            format!(
                "(let (({} (thunk (lambda () {})))) {})",
                gen_var(var),
                gen_expr(input),
                gen_expr(output),
            )
        }

        Expr::Cons(cons, args) => {
            let args = args.iter().map(gen_expr).collect::<Vec<_>>().join(" ");

            format!(
                "(make-cons {} (vector {args}))",
                gen_cons(cons),
            )
        }

        Expr::Case(input, case, default) => {
            let mut output = default
                .as_deref()
                .map(gen_expr)
                .unwrap_or_else(|| String::from("'error"));

            match case {
                Case::Cons(alts) => {
                    for (cons, args, expr) in alts.iter().rev() {
                        let expr =
                            args.iter()
                                .enumerate()
                                .rfold(gen_expr(expr), |expr, (i, arg)| {
                                    format!(
                                        "(let (({} (cons-field scrutinee {i}))) {})",
                                        gen_var(arg),
                                        expr,
                                    )
                                });

                        output = format!(
                            "(if (cons-is? {} scrutinee) {} {})",
                            gen_cons(cons),
                            expr,
                            output,
                        );
                    }
                }

                Case::Lit(alts) => {
                    for (lit, expr) in alts.iter().rev() {
                        output = format!(
                            "(if (equal? scrutinee {}) {} {})",
                            gen_lit(lit),
                            gen_expr(expr),
                            output
                        )
                    }
                }
            }

            format!(
                "(let ((scrutinee (force {}))) {})",
                gen_expr(input),
                output,
            )
        }

        Expr::Prim(prim) => match prim {
            Prim::Extern(name) => format!("(extern \"{name}\")"),

            Prim::Un(prim, expr) => {
                let prim = match prim {
                    UnPrim::IoPure => "io-pure",
                    UnPrim::StrLen => "str-len",

                    UnPrim::Str(PrimNum::Int) => "str-int",
                    UnPrim::Str(PrimNum::Real) => "str-real",
                };

                format!("({prim} {})", gen_expr(expr))
            }

            Prim::Bin(prim, lhs, rhs) => {
                let lhs = gen_expr(lhs);
                let rhs = gen_expr(rhs);

                let prim = match prim {
                    BinPrim::Trace => "trace",
                    BinPrim::IoBind => "io-bind",
                    BinPrim::StrConcat => "str-concat",
                    BinPrim::StrTake => "str-take",
                    BinPrim::StrSkip => "str-skip",
                    BinPrim::StrChar => "str-char",
                    BinPrim::StrFind => "str-find",
                    BinPrim::StrCmp => "str-cmp",

                    BinPrim::Add(PrimNum::Int) => "int-add",
                    BinPrim::Sub(PrimNum::Int) => "int-sub",
                    BinPrim::Mul(PrimNum::Int) => "int-mul",
                    BinPrim::Div(PrimNum::Int) => "int-div",

                    BinPrim::Add(PrimNum::Real) => "real-add",
                    BinPrim::Sub(PrimNum::Real) => "real-sub",
                    BinPrim::Mul(PrimNum::Real) => "real-mul",
                    BinPrim::Div(PrimNum::Real) => "real-div",
                };

                format!("({prim} {lhs} {rhs})")
            }
        },

        Expr::Note(_, expr) => gen_expr(expr),
    }
}
