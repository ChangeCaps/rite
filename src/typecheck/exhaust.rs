use std::{fmt, iter};

use crate::{
    rename,
    typecheck::State,
    var::{Ident, Operator, Var},
};

#[derive(Clone, Debug)]
pub struct Matrix {
    rows: Vec<Row>,
}

#[derive(Clone, Debug)]
pub struct Row {
    pats: Vec<Pat>,
}

#[derive(Clone, Debug)]
pub enum Pat {
    Wild,
    Cons(Var, Var, Vec<Pat>),
}

impl Matrix {
    pub fn new() -> Self {
        Self { rows: Vec::new() }
    }

    pub fn push(&mut self, row: Row) {
        self.rows.push(row);
    }

    fn specialize(self, cons: &Var, arity: usize) -> Self {
        Self {
            rows: self
                .rows
                .into_iter()
                .filter_map(|row| row.specialize(cons, arity))
                .collect(),
        }
    }

    fn type_(&self) -> Option<Var> {
        self.rows.iter().find_map(|row| match row.pats.last() {
            Some(Pat::Cons(type_, ..)) => Some(type_.clone()),
            _ => None,
        })
    }

    fn default(self) -> Self {
        Self {
            rows: self
                .rows
                .into_iter()
                .filter_map(|row| row.is_default().then(|| row.tail()).flatten())
                .collect(),
        }
    }
}

impl Row {
    fn is_default(&self) -> bool {
        self.pats.last().is_some_and(|pat| matches!(pat, Pat::Wild))
    }

    fn tail(mut self) -> Option<Self> {
        self.pats.pop()?;
        Some(self)
    }

    fn specialize(mut self, cons: &Var, arity: usize) -> Option<Self> {
        match self.pats.pop()? {
            Pat::Wild => {
                self.pats.extend(iter::repeat_n(Pat::Wild, arity));
            }

            Pat::Cons(_, var, args) if var == *cons => {
                self.pats.extend(args.into_iter().rev());
            }

            Pat::Cons(..) => return None,
        }

        Some(self)
    }
}

impl fmt::Display for Pat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn paren(input: String, p: u8, q: u8) -> String {
            match p > q {
                true => format!("({input})"),
                false => input,
            }
        }

        fn recurse(pat: &Pat, p: u8) -> String {
            match pat {
                Pat::Wild => String::from("_"),
                Pat::Cons(_, var, args) => {
                    if args.is_empty() {
                        return var.to_string();
                    }

                    if let Some(operator) = var.ident().and_then(Ident::as_operator) {
                        if let Operator::Comma = operator {
                            return format!(
                                "{}{} {}",
                                recurse(&args[0], operator.precedence()),
                                operator.as_str(),
                                recurse(&args[1], operator.precedence())
                            );
                        }

                        return format!(
                            "{}{} {}",
                            recurse(&args[0], operator.precedence()),
                            operator.as_str(),
                            recurse(&args[1], operator.precedence())
                        );
                    }

                    let mut items = vec![var.to_string()];
                    items.extend(args.iter().map(|pat| recurse(pat, 101)));

                    paren(items.join(" "), p, 100)
                }
            }
        }

        f.write_str(&recurse(self, 0))
    }
}

impl State<'_> {
    pub fn row(&self, pat: &rename::Pat) -> Row {
        Row {
            pats: vec![self.exhaust_pat(pat)],
        }
    }

    fn exhaust_pat(&self, pat: &rename::Pat) -> Pat {
        match pat {
            rename::Pat::Bind(..) | rename::Pat::Error(..) => Pat::Wild,
            rename::Pat::Cons(pat) => {
                let args = pat.args.iter().map(|pat| self.exhaust_pat(pat)).collect();
                Pat::Cons(pat.type_.clone(), pat.var.clone(), args)
            }
        }
    }

    pub fn is_useful(&self, matrix: Matrix, row: Row) -> bool {
        let Some(pat) = row.pats.last().cloned() else {
            return matrix.rows.iter().all(|row| !row.pats.is_empty());
        };

        match pat {
            Pat::Cons(_, cons, args) => {
                let matrix = matrix.specialize(&cons, args.len());
                let row = row.specialize(&cons, args.len()).unwrap();
                self.is_useful(matrix, row)
            }

            Pat::Wild => match matrix.type_() {
                Some(type_) => self.tycx.get_tuple_conses(&type_).iter().any(|cons| {
                    let matrix = matrix.clone().specialize(&cons.var, cons.args.len());
                    let row = row.clone().specialize(&cons.var, cons.args.len()).unwrap();
                    self.is_useful(matrix, row)
                }),

                None => {
                    let matrix = matrix.default();
                    let row = row.tail().unwrap();
                    self.is_useful(matrix, row)
                }
            },
        }
    }

    pub fn exhaust(&self, matrix: Matrix) -> Vec<Pat> {
        fn recurse(state: &State<'_>, matrix: Matrix) -> Vec<Vec<Pat>> {
            match matrix.type_() {
                Some(type_) => state
                    .tycx
                    .get_tuple_conses(&type_)
                    .iter()
                    .flat_map(|cons| {
                        let matrix = matrix.clone().specialize(&cons.var, cons.args.len());

                        recurse(state, matrix).into_iter().map(|mut stack| {
                            let args = stack.drain(stack.len() - cons.args.len()..).rev().collect();

                            stack.push(Pat::Cons(
                                type_.clone(),
                                cons.var.clone(),
                                args,
                            ));

                            stack
                        })
                    })
                    .collect(),

                None => {
                    if matrix.rows.iter().any(|row| row.pats.is_empty()) {
                        Vec::new()
                    } else if matrix.rows.is_empty() {
                        vec![vec![Pat::Wild]]
                    } else {
                        recurse(state, matrix.default())
                            .into_iter()
                            .map(|mut stack| {
                                stack.push(Pat::Wild);
                                stack
                            })
                            .collect()
                    }
                }
            }
        }

        recurse(self, matrix)
            .into_iter()
            .map(|mut pats| pats.pop().unwrap())
            .collect()
    }
}
