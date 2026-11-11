use std::{
    collections::{HashMap, HashSet},
    hash::{DefaultHasher, Hash, Hasher},
};

use crate::{
    diagnostic::{Diagnostic, Span},
    typecheck::{Cons, Expr, Kind, Ty},
    var::{Unique, Var},
};

pub type Kinds = HashMap<Unique, Kind>;

pub struct TyCx {
    pub types:  HashMap<Unique, Type>,
    pub traits: HashMap<Unique, Trait>,
    cache:      HashSet<u64>,
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
            types:  HashMap::new(),
            traits: HashMap::new(),
            cache:  HashSet::new(),
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

    pub fn set_inferrable(&self, kinds: &mut Kinds, ty: &Ty, inferrable: bool) {
        match ty {
            Ty::Var(var) if let Some(ty) = Self::subst_shallow(kinds, var).cloned() => {
                self.set_inferrable(kinds, &ty, inferrable);
            }

            Ty::Var(var) => kinds.entry(var.unique()).or_default().inferrable = inferrable,

            Ty::Cons(_, args) => {
                for arg in args {
                    self.set_inferrable(kinds, arg, inferrable);
                }
            }
        }
    }

    pub fn find_impl(&self, kinds: &Kinds, trait_: &Var, ty: &Ty) -> Option<&Impl> {
        self.traits[&trait_.unique()]
            .impls
            .iter()
            .find(|impl_| self.matches(kinds, &impl_.input, ty))
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

    pub fn get_var(&mut self, kinds: &Kinds, var: Var) -> Ty {
        if let Some(subst) = Self::subst_shallow(kinds, &var) {
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

    pub fn instantiate(&self, kinds: &mut Kinds, mut subst: HashMap<Unique, Ty>, ty: Ty) -> Ty {
        fn recurse(kinds: &mut Kinds, subst: &mut HashMap<Unique, Ty>, ty: Ty) -> Ty {
            match ty {
                Ty::Var(var) if let Some(subst) = subst.get(&var.unique()) => subst.clone(),

                Ty::Var(var) => {
                    let new = Var::from(var.ident().cloned());

                    if let Some(mut kind) = kinds.get(&var.unique()).cloned() {
                        kind.inferrable = true;
                        kinds.insert(new.unique(), kind);
                    }

                    let new = Ty::Var(new);
                    subst.insert(var.unique(), new.clone());
                    new
                }

                Ty::Cons(con, args) => {
                    let args = args
                        .into_iter()
                        .map(|arg| recurse(kinds, subst, arg))
                        .collect();

                    Ty::Cons(con, args)
                }
            }
        }

        recurse(kinds, &mut subst, ty)
    }

    fn arity(kinds: &Kinds, ty: &Ty) -> usize {
        match ty {
            Ty::Var(var) => kinds
                .get(&var.unique())
                .and_then(|kind| kind.arity)
                .unwrap_or(0),

            Ty::Cons(cons, args) => match cons {
                Cons::Type(_, arity) => *arity,
                Cons::App => Self::arity(kinds, &args[0]) - 1,

                _ => 0,
            },
        }
    }

    pub fn require(
        &mut self,
        kinds: &mut Kinds,
        ty: &Ty,
        kind: &Kind,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let mut hasher = DefaultHasher::new();

        ty.hash(&mut hasher);
        kind.hash(&mut hasher);

        let hash = hasher.finish();

        if !self.cache.insert(hash) {
            return Ok(());
        }

        match ty {
            Ty::Var(var) => {
                if let Some(ty) = Self::subst_shallow(kinds, var).cloned() {
                    return self.require(kinds, &ty, kind, span);
                }

                kinds
                    .entry(var.unique())
                    .or_default()
                    .intersect(kind, var, span)
            }

            Ty::Cons(..) => {
                if let Some(expected) = kind.arity
                    && let actual = Self::arity(kinds, ty)
                    && actual != expected
                {
                    return Err(
                        Diagnostic::error("kind mismatch").label(
                            span,
                            format!(
                                "expected type with kind `{}` but found `{}` which has kind `{}`",
                                Kind::format_arity(expected),
                                self.format_ty(kinds, ty),
                                Kind::format_arity(actual),
                            ),
                        ),
                    );
                }

                for trait_ in &kind.traits {
                    let Some(impl_) = self.find_impl(kinds, trait_, ty) else {
                        let diagnostic = Diagnostic::error("kind mismatch").label(
                            span,
                            format!(
                                "`{}` does not implement `{}`",
                                self.format_ty(kinds, ty),
                                trait_,
                            ),
                        );

                        return Err(diagnostic);
                    };

                    if let Ty::Var(ref var) = impl_.input
                        && let Some(kind) = kinds.get(&var.unique()).cloned()
                    {
                        self.require(kinds, ty, &kind, span)?;
                    }
                }

                Ok(())
            }
        }
    }

    pub fn unify(
        &mut self,
        kinds: &mut Kinds,
        lhs: &Ty,
        rhs: &Ty,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let mut hasher = DefaultHasher::new();

        lhs.hash(&mut hasher);
        rhs.hash(&mut hasher);

        let hash = hasher.finish();

        if !self.cache.insert(hash) {
            return Ok(());
        }

        if lhs == rhs {
            return Ok(());
        }

        match (lhs, rhs) {
            (Ty::Var(lhs), rhs) if let Some(lhs) = Self::subst_shallow(kinds, lhs).cloned() => {
                self.unify(kinds, &lhs, rhs, span)
            }

            (lhs, Ty::Var(rhs)) if let Some(rhs) = Self::subst_shallow(kinds, rhs).cloned() => {
                self.unify(kinds, lhs, &rhs, span)
            }

            (Ty::Var(var), ty) | (ty, Ty::Var(var))
                if kinds.get(&var.unique()).is_none_or(|kind| kind.inferrable) =>
            {
                if let Some(kind) = kinds.get(&var.unique()).cloned() {
                    self.require(kinds, ty, &kind, span)?;
                }

                let kind = kinds.entry(var.unique()).or_default();

                debug_assert!(kind.subst.is_none());
                kind.subst = Some(ty.clone());

                Ok(())
            }

            (Ty::Cons(lhs_cons, lhs_args), Ty::Cons(rhs_cons, rhs_args))
                if lhs_cons == rhs_cons =>
            {
                debug_assert_eq!(lhs_args.len(), rhs_args.len());

                for (lhs, rhs) in lhs_args.iter().zip(rhs_args) {
                    self.unify(kinds, lhs, rhs, span)?;
                }

                Ok(())
            }

            (_, _) => Err(
                Diagnostic::error("type mismatch").label(
                    span,
                    format!(
                        "expected `{}` but found `{}`",
                        self.format_ty(kinds, lhs),
                        self.format_ty(kinds, rhs),
                    ),
                ),
            ),
        }
    }

    pub fn matches(&self, kinds: &Kinds, lhs: &Ty, rhs: &Ty) -> bool {
        if lhs == rhs {
            return true;
        };

        match (lhs, rhs) {
            (Ty::Var(lhs), rhs) if let Some(lhs) = Self::subst_shallow(kinds, lhs) => {
                self.matches(kinds, lhs, rhs)
            }

            (lhs, Ty::Var(rhs)) if let Some(rhs) = Self::subst_shallow(kinds, rhs) => {
                self.matches(kinds, lhs, rhs)
            }

            (Ty::Var(_), _) | (_, Ty::Var(_)) => true,

            (Ty::Cons(lhs_cons, lhs_args), Ty::Cons(rhs_cons, rhs_args)) => {
                lhs_cons == rhs_cons
                    && lhs_args
                        .iter()
                        .zip(rhs_args)
                        .all(|(lhs, rhs)| self.matches(kinds, lhs, rhs))
            }
        }
    }

    fn subst_shallow<'a>(kinds: &'a Kinds, var: &Var) -> Option<&'a Ty> {
        kinds.get(&var.unique())?.subst.as_ref()
    }

    pub fn format_ty(&self, kinds: &Kinds, ty: &Ty) -> String {
        fn fmt_paren(input: String, p: u8, q: u8) -> String {
            match p > q {
                true => format!("({input})"),
                false => input,
            }
        }

        fn fmt_ty(kinds: &Kinds, ty: &Ty, p: u8) -> String {
            match ty {
                Ty::Var(var) => match TyCx::subst_shallow(kinds, var) {
                    Some(ty) => fmt_ty(kinds, ty, p),
                    None => var.to_string(),
                },

                Ty::Cons(cons, args) => match cons {
                    Cons::Str => String::from("str"),
                    Cons::Int => String::from("int"),
                    Cons::Real => String::from("real"),

                    Cons::App => fmt_paren(
                        format!(
                            "{} {}",
                            fmt_ty(kinds, &args[0], 1),
                            fmt_ty(kinds, &args[1], 2)
                        ),
                        p,
                        1,
                    ),

                    Cons::Lam => fmt_paren(
                        format!(
                            "{} -> {}",
                            fmt_ty(kinds, &args[0], 1),
                            fmt_ty(kinds, &args[1], 0),
                        ),
                        p,
                        0,
                    ),

                    Cons::Type(var, _) => var.to_string(),
                },
            }
        }

        fmt_ty(kinds, ty, 0)
    }
}
