use std::collections::HashMap;

use crate::{
    diagnostic::{Diagnostic, Reporter, Span},
    rename,
    var::{Unique, Var},
};

mod expr;
mod kind;
mod pat;
mod ty;
mod tycx;

pub use expr::*;
pub use kind::*;
pub use pat::*;
pub use ty::*;
pub use tycx::*;

pub struct Module {
    pub tycx:    TyCx,
    pub kinds:   Kinds,
    pub defs:    HashMap<Unique, Def>,
    pub methods: HashMap<Unique, TraitMethod>,
}

pub struct Def {
    pub var:  Var,
    pub expr: Expr,
    pub span: Span,
}

pub struct TraitMethod {
    pub var:   Var,
    pub input: Var,
    pub span:  Span,
}

pub fn typecheck(reporter: &dyn Reporter, module: rename::Unit) -> Module {
    let state = State::new(reporter);
    state.module(module)
}

struct State<'a> {
    reporter: &'a dyn Reporter,
    tycx:     TyCx,
    kinds:    Kinds,
    defs:     HashMap<Unique, Def>,
    methods:  HashMap<Unique, TraitMethod>,
}

impl<'a> State<'a> {
    fn new(reporter: &'a dyn Reporter) -> Self {
        Self {
            reporter,
            tycx: TyCx::new(),
            kinds: Kinds::new(),
            defs: HashMap::new(),
            methods: HashMap::new(),
        }
    }

    fn module(mut self, mut module: rename::Unit) -> Module {
        for type_ in module.types.into_values() {
            self.add_type(type_);
        }

        for trait_ in module.traits.into_values() {
            self.add_trait(trait_);
        }

        for def in module.defs.values_mut() {
            if let Some(rhs) = def.ty.take() {
                let lhs = self.get_ty_var(def.var.clone());
                let rhs = self.ty(rhs);

                self.set_inferrable(&rhs, false);
                self.unify(&lhs, &rhs, def.span);
            }
        }

        for impl_ in module.impls {
            self.add_impl(impl_);
        }

        for def in module.defs.into_values() {
            self.add_def(def);
        }

        Module {
            tycx:    self.tycx,
            kinds:   self.kinds,
            defs:    self.defs,
            methods: self.methods,
        }
    }

    fn add_type(&mut self, type_: rename::Type) {
        let ty = Ty::Cons(
            Cons::Type(type_.var.clone(), type_.input.len()),
            Vec::new(),
        );

        let ty = type_.input.iter().cloned().fold(ty, |ty, input| {
            let input = self.get_ty_var(input);
            Ty::Cons(Cons::App, vec![ty, input])
        });

        let cons = type_
            .cons
            .into_iter()
            .map(|cons| {
                let args = cons
                    .args
                    .into_iter()
                    .map(|ty| self.ty(ty))
                    .collect::<Vec<_>>();

                let ty = args.iter().cloned().rfold(ty.clone(), |ty, arg| {
                    Ty::Cons(Cons::Lam, vec![arg, ty])
                });

                self.add_subst(&cons.var, ty);

                TypeCons {
                    var: cons.var,
                    args,
                    span: cons.span,
                }
            })
            .collect();

        self.tycx.add_type(
            type_.var.clone(),
            type_.input.clone(),
            cons,
            type_.span,
        );
    }

    fn add_trait(&mut self, trait_: rename::Trait) {
        let input = self.get_ty_var(trait_.input.clone());

        let kind = Kind {
            traits: vec![trait_.var.clone()],
            ..Default::default()
        };

        self.require_kind(&input, &kind, trait_.span);

        if let Some(kind) = trait_.kind {
            let kind = self.kind(kind);
            self.require_kind(&input, &kind, trait_.span);
        }

        for method in trait_.methods {
            let ty = self.ty(method.ty);
            self.add_subst(&method.var, ty);

            self.methods.insert(
                method.var.unique(),
                TraitMethod {
                    var:   trait_.var.clone(),
                    input: trait_.input.clone(),
                    span:  method.span,
                },
            );
        }

        self.tycx.add_trait(trait_.var, trait_.input, trait_.span);
    }

    fn add_impl(&mut self, impl_: rename::Impl) {
        let trait_ = self.tycx.get_trait(&impl_.var);

        let var = trait_.input.clone();
        let input = self.ty(impl_.input);

        if let Some(found) = self.find_impl(&impl_.var, &input) {
            let diagnostic = Diagnostic::error("conflicting implementation")
                .label(impl_.span, "first implementation")
                .label(found.span, "second implementation");

            self.reporter.report(diagnostic);

            return;
        }

        let subst = HashMap::from([(var.unique(), input.clone())]);

        let index = self.tycx.add_impl(&impl_.var, input.clone(), impl_.span);

        let mut members = Vec::new();

        for member in impl_.methods {
            let ty = self.get_ty_var(member.var.clone());
            let ty = self.instantiate(subst.clone(), ty);
            self.set_inferrable(&ty, false);

            let expr = self.expr_up(member.expr, ty.clone());

            members.push(ImplMethod {
                var: member.var,
                expr,
                span: member.span,
            });
        }

        self.tycx.set_members(&impl_.var, index, members);

        {
            let var = self.get_ty_var(var.clone());
            let var = self.instantiate(HashMap::new(), var);
            self.unify(&var, &input, impl_.span);
        }
    }

    fn add_def(&mut self, def: rename::Def) {
        let expr = self.expr_down(def.expr);

        let var = self.get_ty_var(def.var.clone());
        self.unify(&var, expr.ty(), def.span);

        let def = Def {
            var: def.var,
            expr,
            span: def.span,
        };

        self.defs.insert(def.var.unique(), def);
    }

    fn add_subst(&mut self, var: &Var, ty: Ty) {
        let kind = self.kinds.entry(var.unique()).or_default();

        debug_assert!(kind.subst.is_none());
        kind.subst = Some(ty);
    }

    fn find_impl(&self, trait_: &Var, ty: &Ty) -> Option<&Impl> {
        self.tycx.find_impl(&self.kinds, trait_, ty)
    }

    fn set_inferrable(&mut self, ty: &Ty, inferrable: bool) {
        self.tycx.set_inferrable(&mut self.kinds, ty, inferrable);
    }

    fn get_ty_var(&mut self, var: Var) -> Ty {
        self.tycx.get_var(&self.kinds, var)
    }

    fn instantiate(&mut self, subst: HashMap<Unique, Ty>, ty: Ty) -> Ty {
        self.tycx.instantiate(&mut self.kinds, subst, ty)
    }

    fn require_kind(&mut self, lhs: &Ty, rhs: &Kind, span: Span) {
        if let Err(diagnostic) = self.tycx.require(&mut self.kinds, lhs, rhs, span) {
            self.reporter.report(diagnostic);
        }
    }

    fn unify(&mut self, lhs: &Ty, rhs: &Ty, span: Span) {
        if let Err(diagnostic) = self.tycx.unify(&mut self.kinds, lhs, rhs, span) {
            self.reporter.report(diagnostic);
        }
    }
}
