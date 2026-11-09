use std::collections::{HashMap, HashSet, hash_map::Entry};

use crate::{
    diagnostic::{Diagnostic, Span},
    typecheck::{Cons, Expr, Kind, Ty},
    var::{Unique, Var},
};

pub type Subst = HashMap<Unique, Ty>;
pub type Kinds = HashMap<Unique, Kind>;

pub struct TyCx {
    pub types:   HashMap<Unique, Type>,
    pub traits:  HashMap<Unique, Trait>,
    pub generic: HashSet<Unique>,
}

pub struct Type {
    pub input: Vec<Var>,
    pub cons:  Vec<TypeCons>,
    pub span:  Span,
}

pub struct TypeCons {
    pub var:  Var,
    pub args: Vec<Ty>,
    pub span: Span,
}

pub struct Trait {
    pub input: Var,
    pub impls: Vec<Impl>,
    pub span:  Span,
}

pub struct Impl {
    pub input:   Ty,
    pub methods: Vec<ImplMethod>,
    pub span:    Span,
}

pub struct ImplMethod {
    pub var:  Var,
    pub expr: Expr,
    pub span: Span,
}

impl Default for TyCx {
    fn default() -> Self {
        Self::new()
    }
}

impl TyCx {
    pub fn new() -> Self {
        Self {
            types:   HashMap::new(),
            traits:  HashMap::new(),
            generic: HashSet::new(),
        }
    }

    pub fn add_trait(&mut self, var: Var, input: Var, span: Span) {
        let trait_ = Trait {
            input,
            impls: Vec::new(),
            span,
        };

        self.traits.insert(var.unique(), trait_);
    }

    pub fn add_type(&mut self, var: Var, input: Vec<Var>, cons: Vec<TypeCons>, span: Span) {
        self.types.insert(var.unique(), Type { input, cons, span });
    }

    pub fn add_impl(&mut self, trait_: &Var, input: Ty, span: Span) -> usize {
        let impl_ = Impl {
            input,
            methods: Vec::new(),
            span,
        };

        let trait_ = self.traits.get_mut(&trait_.unique()).unwrap();
        trait_.impls.push(impl_);
        trait_.impls.len() - 1
    }

    pub fn set_members(&mut self, trait_: &Var, index: usize, members: Vec<ImplMethod>) {
        let trait_ = self.traits.get_mut(&trait_.unique()).unwrap();
        trait_.impls[index].methods = members;
    }

    pub fn make_generic(&mut self, subst: &Subst, ty: &Ty) {
        match ty {
            Ty::Var(var) if let Some(ty) = subst.get(&var.unique()).cloned() => {
                self.make_generic(subst, &ty);
            }

            Ty::Var(var) => {
                self.generic.insert(var.unique());
            }

            Ty::Cons(_, args) => {
                for arg in args {
                    self.make_generic(subst, arg);
                }
            }
        }
    }

    pub fn find_impl(&self, subst: &Subst, trait_: &Var, ty: &Ty) -> Option<&Impl> {
        self.traits[&trait_.unique()]
            .impls
            .iter()
            .find(|impl_| self.check(subst, &impl_.input, ty))
    }

    #[track_caller]
    pub fn get_type(&self, var: &Var) -> &Type {
        self.types
            .get(&var.unique())
            .expect("should only be called with valid `var`")
    }

    #[track_caller]
    pub fn get_cons(&self, type_: &Var, var: &Var) -> &TypeCons {
        self.get_type(type_)
            .cons
            .iter()
            .find(|cons| cons.var == *var)
            .expect("should only be called with var `type_` and `var`")
    }

    #[track_caller]
    pub fn get_trait(&self, var: &Var) -> &Trait {
        self.traits
            .get(&var.unique())
            .expect("should only be called with valid `var`")
    }

    pub fn get_var(&mut self, subst: &Subst, var: Var) -> Ty {
        if let Some(subst) = subst.get(&var.unique()) {
            return subst.clone();
        }

        if let Some(type_) = self.types.get(&var.unique()) {
            return Ty::Cons(
                Cons::Type(var.clone(), type_.input.len()),
                Vec::new(),
            );
        }

        Ty::Var(var)
    }

    pub fn instantiate(&mut self, kinds: &mut Kinds, mut subst: HashMap<Unique, Ty>, ty: Ty) -> Ty {
        fn recurse(
            tycx: &mut TyCx,
            kinds: &mut Kinds,
            subst: &mut HashMap<Unique, Ty>,
            ty: Ty,
        ) -> Ty {
            match ty {
                Ty::Var(var) if let Some(subst) = subst.get(&var.unique()) => subst.clone(),

                Ty::Var(var) => {
                    let new = Var::from(var.ident().cloned());

                    if let Some(kind) = kinds.get(&var.unique()).cloned() {
                        kinds.insert(new.unique(), kind);
                    }

                    let new = Ty::Var(new);
                    subst.insert(var.unique(), new.clone());
                    new
                }

                Ty::Cons(con, args) => {
                    let args = args
                        .into_iter()
                        .map(|arg| recurse(tycx, kinds, subst, arg))
                        .collect();

                    Ty::Cons(con, args)
                }
            }
        }

        recurse(self, kinds, &mut subst, ty)
    }

    pub fn unify_kind(
        &self,
        kinds: &mut Kinds,
        subst: &Subst,
        lhs: &Ty,
        rhs: &Kind,
        span: Span,
    ) -> Result<(), Diagnostic> {
        match lhs {
            Ty::Var(var) => match subst.get(&var.unique()).cloned() {
                Some(lhs) => self.unify_kind(kinds, subst, &lhs, rhs, span),

                None => {
                    match kinds.entry(var.unique()) {
                        Entry::Occupied(mut entry) => {
                            entry.get_mut().inter(rhs.clone());
                        }

                        Entry::Vacant(entry) => {
                            entry.insert(rhs.clone());
                        }
                    }

                    Ok(())
                }
            },

            Ty::Cons(cons, args) => match rhs {
                Kind::Inter(rhs) => {
                    for rhs in rhs {
                        self.unify_kind(kinds, subst, lhs, rhs, span)?;
                    }

                    Ok(())
                }

                Kind::Arity(arity) => {
                    if matches!(cons, Cons::Type(_, a) if a == arity) {
                        return Ok(());
                    }

                    if matches!(cons, Cons::App) {
                        return self.unify_kind(
                            kinds,
                            subst,
                            &args[0],
                            &Kind::Arity(arity + 1),
                            span,
                        );
                    }

                    let kind = (0..=*arity).map(|_| "*").collect::<Vec<_>>().join(" -> ");

                    Err(
                        Diagnostic::error("kind mismatch").label(
                            span,
                            format!(
                                "expected `{kind}` but found `{}`",
                                self.format_ty(subst, lhs),
                            ),
                        ),
                    )
                }

                Kind::Trait(var) => {
                    if self.find_impl(subst, var, lhs).is_none() {
                        let diagnostic = Diagnostic::error("kind mismatch").label(
                            span,
                            format!(
                                "`{}` does not implement `{}`",
                                self.format_ty(subst, lhs),
                                var,
                            ),
                        );

                        return Err(diagnostic);
                    }

                    Ok(())
                }
            },
        }
    }

    pub fn unify(
        &self,
        kinds: &mut Kinds,
        subst: &mut Subst,
        lhs: &Ty,
        rhs: &Ty,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if lhs == rhs {
            return Ok(());
        }

        match (lhs, rhs) {
            (Ty::Var(lhs), rhs) if let Some(lhs) = subst.get(&lhs.unique()).cloned() => {
                self.unify(kinds, subst, &lhs, rhs, span)
            }

            (lhs, Ty::Var(rhs)) if let Some(rhs) = subst.get(&rhs.unique()).cloned() => {
                self.unify(kinds, subst, lhs, &rhs, span)
            }

            (Ty::Var(var), ty) | (ty, Ty::Var(var)) if !self.generic.contains(&var.unique()) => {
                if let Some(kind) = kinds.get(&var.unique()).cloned() {
                    self.unify_kind(kinds, subst, ty, &kind, span)?;
                }

                subst.insert(var.unique(), ty.clone());

                Ok(())
            }

            (Ty::Cons(lhs_cons, lhs_args), Ty::Cons(rhs_cons, rhs_args))
                if lhs_cons == rhs_cons =>
            {
                debug_assert_eq!(lhs_args.len(), rhs_args.len());

                for (lhs, rhs) in lhs_args.iter().zip(rhs_args) {
                    self.unify(kinds, subst, lhs, rhs, span)?;
                }

                Ok(())
            }

            (_, _) => Err(
                Diagnostic::error("type mismatch").label(
                    span,
                    format!(
                        "expected `{}` but found `{}`",
                        self.format_ty(subst, lhs),
                        self.format_ty(subst, rhs),
                    ),
                ),
            ),
        }
    }

    pub fn check(&self, subst: &Subst, lhs: &Ty, rhs: &Ty) -> bool {
        if lhs == rhs {
            return true;
        };

        match (lhs, rhs) {
            (Ty::Var(lhs), rhs) if let Some(lhs) = subst.get(&lhs.unique()) => {
                self.check(subst, lhs, rhs)
            }

            (lhs, Ty::Var(rhs)) if let Some(rhs) = subst.get(&rhs.unique()) => {
                self.check(subst, lhs, rhs)
            }

            (Ty::Var(_), _) | (_, Ty::Var(_)) => true,

            (Ty::Cons(lhs_cons, lhs_args), Ty::Cons(rhs_cons, rhs_args)) => {
                lhs_cons == rhs_cons
                    && lhs_args
                        .iter()
                        .zip(rhs_args)
                        .all(|(lhs, rhs)| self.check(subst, lhs, rhs))
            }
        }
    }

    pub fn subst(&self, subst: &Subst, mut ty: Ty) -> Ty {
        fn recurse(subst: &Subst, ty: &mut Ty) {
            match ty {
                Ty::Var(var) => {
                    if let Some(sub) = subst.get(&var.unique()) {
                        *ty = sub.clone();
                        recurse(subst, ty);
                    }
                }

                Ty::Cons(_, args) => {
                    for arg in args {
                        recurse(subst, arg);
                    }
                }
            }
        }

        recurse(subst, &mut ty);
        ty
    }

    pub fn format_ty(&self, subst: &Subst, ty: &Ty) -> String {
        fn fmt_paren(input: String, p: u8, q: u8) -> String {
            match p > q {
                true => format!("({input})"),
                false => input,
            }
        }

        fn fmt_ty(tycx: &TyCx, subst: &Subst, ty: &Ty, p: u8) -> String {
            match ty {
                Ty::Var(var) => match subst.get(&var.unique()) {
                    Some(ty) => fmt_ty(tycx, subst, ty, p),
                    None => var.to_string(),
                },

                Ty::Cons(cons, args) => match cons {
                    Cons::App => fmt_paren(
                        format!(
                            "{} {}",
                            fmt_ty(tycx, subst, &args[0], 1),
                            fmt_ty(tycx, subst, &args[1], 2)
                        ),
                        p,
                        1,
                    ),

                    Cons::Lam => fmt_paren(
                        format!(
                            "{} -> {}",
                            fmt_ty(tycx, subst, &args[0], 1),
                            fmt_ty(tycx, subst, &args[1], 0),
                        ),
                        p,
                        0,
                    ),

                    Cons::Type(var, _) => var.to_string(),
                },
            }
        }

        fmt_ty(self, subst, ty, 0)
    }
}
