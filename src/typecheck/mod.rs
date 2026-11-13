use std::collections::HashMap;

use crate::{
    diagnostic::{Diagnostic, Reporter, Span},
    rename,
    var::{Unique, Var},
};

mod exhaust;
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

pub struct Unit {
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

pub fn typecheck(reporter: &dyn Reporter, unit: rename::Unit) -> Unit {
    let state = State::new(reporter);
    state.unit(unit)
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

    fn unit(mut self, mut unit: rename::Unit) -> Unit {
        for type_ in unit.types.into_values() {
            self.add_type(type_);
        }

        for trait_ in unit.traits.into_values() {
            self.add_trait(trait_);
        }

        for def in unit.defs.values_mut() {
            if let Some(rhs) = def.ty.take() {
                let lhs = Ty::Var(def.var.clone());
                let rhs = self.ty(rhs);
                self.require_arity(&rhs, Arity::exact(0), def.span);

                self.set_inferrable(&rhs, false);
                self.unify(&lhs, &rhs, def.span);
            }
        }

        for impl_ in unit.impls {
            self.add_impl(impl_);
        }

        for def in unit.defs.into_values() {
            self.add_def(def);
        }

        Unit {
            tycx:    self.tycx,
            kinds:   self.kinds,
            defs:    self.defs,
            methods: self.methods,
        }
    }

    fn add_type(&mut self, type_: rename::Type) {
        let ty = type_
            .input
            .iter()
            .cloned()
            .map(Ty::Var)
            .fold(Ty::Type(type_.var.clone()), Ty::app);

        match type_.cons {
            rename::TypeCons::Rec(cons) => {
                let fields = cons
                    .fields
                    .into_iter()
                    .map(|field| RecordField {
                        name: field.name,
                        ty:   self.ty(field.ty),
                        span: field.span,
                    })
                    .collect();

                self.tycx.add_type(
                    type_.var.clone(),
                    type_.input.clone(),
                    TypeCons::Record(RecordCons {
                        var: cons.var,
                        fields,
                        span: cons.span,
                    }),
                    type_.span,
                );
            }

            rename::TypeCons::Tuple(cons) => {
                let cons = cons
                    .into_iter()
                    .map(|cons| self.add_tuple_cons(ty.clone(), cons))
                    .collect();

                self.tycx.add_type(
                    type_.var.clone(),
                    type_.input.clone(),
                    TypeCons::Tuple(cons),
                    type_.span,
                );
            }
        }
    }

    fn add_tuple_cons(&mut self, ty: Ty, cons: rename::TupleCons) -> TupleCons {
        let args = cons
            .args
            .into_iter()
            .map(|ty| {
                let span = ty.span();
                let arg = self.ty(ty);
                self.require_arity(&arg, Arity::exact(0), span);
                arg
            })
            .collect::<Vec<_>>();

        let ty = args.iter().cloned().rfold(ty, |ty, arg| Ty::lam(arg, ty));

        self.add_subst(&cons.var, ty);

        TupleCons {
            var: cons.var,
            args,
            span: cons.span,
        }
    }

    fn add_trait(&mut self, trait_: rename::Trait) {
        let input = Ty::Var(trait_.input.clone());

        let kind = Kind {
            traits: vec![trait_.var.clone()],
            ..Default::default()
        };

        self.require(&input, &kind, trait_.span);

        if let Some(kind) = trait_.kind {
            let kind = self.kind(kind);
            self.require(&input, &kind, trait_.span);
        }

        for method in trait_.methods {
            let ty = self.ty(method.ty);

            self.require_arity(&ty, Arity::exact(0), method.span);
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

        let mut subst = HashMap::from([(var.unique(), input.clone())]);

        let index = self.tycx.add_impl(&impl_.var, input.clone(), impl_.span);

        let mut members = Vec::new();

        for member in impl_.methods {
            let mut ty = Ty::Var(member.var.clone());
            self.instantiate(&mut subst, &mut ty);
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
            let mut var = Ty::Var(var.clone());
            self.instantiate(&mut HashMap::new(), &mut var);
            self.unify(&input, &var, impl_.span);
        }
    }

    fn add_def(&mut self, def: rename::Def) {
        let expr = self.expr_down(def.expr);

        let var = Ty::Var(def.var.clone());
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

    fn instantiate(&mut self, subst: &mut HashMap<Unique, Ty>, ty: &mut Ty) {
        self.tycx.instantiate(&mut self.kinds, subst, ty)
    }

    fn instantiate_all(&mut self, subst: &mut HashMap<Unique, Ty>, tys: &mut [Ty]) {
        self.tycx.instantiate_all(&mut self.kinds, subst, tys);
    }

    fn require_arity(&mut self, ty: &Ty, arity: Arity, span: Span) {
        self.require(ty, &Kind::arity(arity), span);
    }

    fn require(&mut self, ty: &Ty, kind: &Kind, span: Span) {
        if let Err(diagnostic) = self.tycx.require(&mut self.kinds, ty, kind, span) {
            self.reporter.report(diagnostic);
        }
    }

    fn unify(&mut self, lhs: &Ty, rhs: &Ty, span: Span) {
        if let Err(diagnostic) = self.tycx.unify(&mut self.kinds, lhs, rhs, span) {
            self.reporter.report(diagnostic);
        }
    }
}
