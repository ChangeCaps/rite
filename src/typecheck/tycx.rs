use std::{
    collections::{HashMap, HashSet},
    hash::{DefaultHasher, Hash, Hasher},
};

use crate::{
    diagnostic::{Diagnostic, Span},
    typecheck::{Expr, Kind, Ty},
    var::{Ident, Unique, Var},
};

pub type Kinds = HashMap<Unique, Kind>;

pub struct TyCx {
    pub types:  HashMap<Unique, Type>,
    pub traits: HashMap<Unique, Trait>,
    cache:      HashSet<u64>,
}

pub struct Type {
    pub args: Vec<Var>,
    pub cons: TypeCons,
    pub span: Span,
}

pub enum TypeCons {
    Record(RecordCons),
    Tuple(Vec<TupleCons>),
}

pub struct RecordCons {
    pub var:    Var,
    pub fields: Vec<RecordField>,
    pub span:   Span,
}

#[derive(Clone)]
pub struct RecordField {
    pub name: Ident,
    pub ty:   Ty,
    pub span: Span,
}

pub struct TupleCons {
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

    pub fn add_type(&mut self, var: Var, input: Vec<Var>, cons: TypeCons, span: Span) {
        self.types.insert(
            var.unique(),
            Type {
                args: input,
                cons,
                span,
            },
        );
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

            Ty::Var(var) => kinds.entry(var.unique()).or_default().infer = inferrable,

            Ty::App(lambda, input) => {
                self.set_inferrable(kinds, lambda, inferrable);
                self.set_inferrable(kinds, input, inferrable);
            }

            _ => {}
        }
    }

    pub fn find_impl(&self, kinds: &Kinds, trait_: &Var, ty: &Ty) -> Option<&Impl> {
        self.traits[&trait_.unique()]
            .impls
            .iter()
            .find(|impl_| self.matches(kinds, &impl_.input, ty))
    }

    pub fn type_ty(&self, var: Var) -> Ty {
        let type_ = self.get_type(&var);

        type_
            .args
            .iter()
            .cloned()
            .map(Ty::Var)
            .fold(Ty::Type(var), Ty::app)
    }

    #[track_caller]
    pub fn get_type(&self, var: &Var) -> &Type {
        self.types
            .get(&var.unique())
            .expect("should only be called with valid `var`")
    }

    #[track_caller]
    pub fn get_tuple_conses(&self, type_: &Var) -> &[TupleCons] {
        match self.get_type(type_).cons {
            TypeCons::Tuple(ref cons) => cons,
            _ => panic!("type must have a tuple constructor"),
        }
    }

    #[track_caller]
    pub fn get_tuple_cons(&self, type_: &Var, var: &Var) -> &TupleCons {
        self.get_tuple_conses(type_)
            .iter()
            .find(|cons| cons.var == *var)
            .expect("type must have the constructor")
    }

    #[track_caller]
    pub fn get_trait(&self, var: &Var) -> &Trait {
        self.traits
            .get(&var.unique())
            .expect("should only be called with valid `var`")
    }

    pub fn instantiate(&self, kinds: &mut Kinds, subst: &mut HashMap<Unique, Ty>, ty: &mut Ty) {
        fn recurse(kinds: &mut Kinds, subst: &mut HashMap<Unique, Ty>, ty: &mut Ty) {
            match ty {
                Ty::Var(var) if let Some(subst) = subst.get(&var.unique()) => {
                    *ty = subst.clone();
                }

                Ty::Var(var) => {
                    let new = Var::from(var.ident().cloned());

                    if let Some(mut kind) = kinds.get(&var.unique()).cloned() {
                        if let Some(ref mut ty) = kind.subst {
                            recurse(kinds, subst, ty);
                        }

                        kind.infer = true;
                        kinds.insert(new.unique(), kind);
                    }

                    let new = Ty::Var(new);
                    subst.insert(var.unique(), new.clone());
                    *ty = new;
                }

                Ty::App(lambda, input) => {
                    recurse(kinds, subst, lambda);
                    recurse(kinds, subst, input);
                }

                _ => {}
            }
        }

        recurse(kinds, subst, ty);
    }

    pub fn instantiate_all(
        &self,
        kinds: &mut Kinds,
        subst: &mut HashMap<Unique, Ty>,
        tys: &mut [Ty],
    ) {
        for ty in tys {
            self.instantiate(kinds, subst, ty);
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

        if let Ty::Var(var) = ty {
            return self.require_var(kinds, var, kind, span);
        }

        for trait_ in &kind.traits {
            if self.find_impl(kinds, trait_, ty).is_none() {
                let diagnostic = Diagnostic::error("kind mismatch").label(
                    span,
                    format!(
                        "`{}` does not implement trait `{}`",
                        self.format_ty(kinds, ty),
                        trait_,
                    ),
                );

                return Err(diagnostic);
            }
        }

        let (ty, args) = ty.reduce();

        for field in &kind.fields {
            if let Ty::Type(type_) = ty
                && let type_ = self.get_type(type_)
                && let TypeCons::Record(ref cons) = type_.cons
                && let Some(type_field) = cons.fields.iter().find(|f| *f.name == field.name)
            {
                let mut subst = type_
                    .args
                    .iter()
                    .map(Var::unique)
                    .zip(args.iter().map(|ty| (*ty).clone()))
                    .collect();

                let mut ty = type_field.ty.clone();
                self.instantiate(kinds, &mut subst, &mut ty);

                self.unify(kinds, &ty, &field.ty, span)?;
            } else {
                return Err(
                    Diagnostic::error("kind mismatch").label(
                        span,
                        format!(
                            "type `{}` does not have field `{}`",
                            self.format_ty(kinds, ty),
                            field.name,
                        ),
                    ),
                );
            }
        }

        Ok(())
    }

    fn require_var(
        &mut self,
        kinds: &mut Kinds,
        var: &Var,
        kind: &Kind,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if let Some(ty) = Self::subst_shallow(kinds, var).cloned() {
            return self.require(kinds, &ty, kind, span);
        }

        let this = kinds.entry(var.unique()).or_default();

        for trait_ in &kind.traits {
            if !(this.traits.contains(trait_) || this.infer) {
                return Err(
                    Diagnostic::error("kind mismatch").label(
                        span,
                        format!(
                            "type `{}` does not implement trait `{}`",
                            self.format_ty(kinds, &Ty::Var(var.clone())),
                            trait_,
                        ),
                    ),
                );
            }

            this.traits.push(trait_.clone());
        }

        let mut unify = Vec::new();

        for field in &kind.fields {
            if let Some(this) = this.fields.iter().find(|this| this.name == field.name) {
                unify.push((this.ty.clone(), field.ty.clone()));
            } else if this.infer {
                this.fields.push(field.clone());
            } else {
                return Err(
                    Diagnostic::error("kind mismatch").label(
                        span,
                        format!(
                            "type `{}` does not have field `{}`",
                            self.format_ty(kinds, &Ty::Var(var.clone())),
                            field.name,
                        ),
                    ),
                );
            }
        }

        for (lhs, rhs) in unify {
            self.unify(kinds, &lhs, &rhs, span)?;
        }

        Ok(())
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

        match (lhs, rhs) {
            (lhs, rhs) if lhs == rhs => Ok(()),

            (Ty::Var(lhs), rhs) if let Some(lhs) = Self::subst_shallow(kinds, lhs).cloned() => {
                self.unify(kinds, &lhs, rhs, span)
            }

            (lhs, Ty::Var(rhs)) if let Some(rhs) = Self::subst_shallow(kinds, rhs).cloned() => {
                self.unify(kinds, lhs, &rhs, span)
            }

            (Ty::Var(var), ty) | (ty, Ty::Var(var))
                if kinds.get(&var.unique()).is_none_or(|kind| kind.infer) =>
            {
                if let Some(kind) = kinds.get(&var.unique()).cloned() {
                    self.require(kinds, ty, &kind, span)?;
                }

                let kind = kinds.entry(var.unique()).or_default();

                debug_assert!(kind.subst.is_none());
                kind.subst = Some(ty.clone());

                Ok(())
            }

            (Ty::App(lhs_lambda, lhs_input), Ty::App(rhs_lambda, rhs_input)) => {
                self.unify(kinds, lhs_lambda, rhs_lambda, span)?;
                self.unify(kinds, lhs_input, rhs_input, span)?;

                Ok(())
            }

            (lhs, rhs) => Err(
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
            (lhs, rhs) if lhs == rhs => true,

            (Ty::Var(lhs), rhs) if let Some(lhs) = Self::subst_shallow(kinds, lhs) => {
                self.matches(kinds, lhs, rhs)
            }

            (lhs, Ty::Var(rhs)) if let Some(rhs) = Self::subst_shallow(kinds, rhs) => {
                self.matches(kinds, lhs, rhs)
            }

            (Ty::Var(..), ..) | (.., Ty::Var(..)) => true,

            (Ty::App(lhs_lambda, lhs_input), Ty::App(rhs_lambda, rhs_input)) => {
                self.matches(kinds, lhs_lambda, rhs_lambda)
                    && self.matches(kinds, lhs_input, rhs_input)
            }

            (_, _) => false,
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

                Ty::Io => String::from("io"),
                Ty::Str => String::from("str"),
                Ty::Int => String::from("int"),
                Ty::Real => String::from("real"),

                Ty::Lam => String::from("(->)"),

                Ty::Type(var) => var.to_string(),

                Ty::App(..) => {
                    let (ty, args) = ty.reduce();

                    match ty {
                        Ty::Lam if args.len() == 2 => {
                            let input = fmt_ty(kinds, args[0], 1);
                            let output = fmt_ty(kinds, args[1], 0);

                            fmt_paren(format!("{input} -> {output}"), p, 0)
                        }

                        _ => {
                            let mut items = args.clone();
                            items.insert(0, ty);

                            fmt_paren(
                                items
                                    .iter()
                                    .map(|ty| fmt_ty(kinds, ty, 2))
                                    .collect::<Vec<_>>()
                                    .join(" "),
                                p,
                                1,
                            )
                        }
                    }
                }
            }
        }

        fmt_ty(kinds, ty, 0)
    }
}
