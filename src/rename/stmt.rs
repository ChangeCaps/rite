use crate::{
    ast,
    diagnostic::{Diagnostic, Span},
    rename::{ErrorExpr, Expr, Kind, State, Ty, ValueKind},
    var::Var,
};

pub struct Def {
    pub ty:   Option<Ty>,
    pub var:  Var,
    pub expr: Expr,
    pub span: Span,
}

pub struct Type {
    pub var:   Var,
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
    pub var:     Var,
    pub input:   Var,
    pub kind:    Option<Kind>,
    pub methods: Vec<TraitMethod>,
    pub span:    Span,
}

pub struct TraitMethod {
    pub ty:   Ty,
    pub var:  Var,
    pub span: Span,
}

pub struct Impl {
    pub var:     Var,
    pub input:   Ty,
    pub methods: Vec<ImplMethod>,
    pub span:    Span,
}

pub struct ImplMethod {
    pub var:  Var,
    pub expr: Expr,
    pub span: Span,
}

impl State<'_> {
    pub fn stmts(&mut self, stmts: Vec<ast::Stmt>) {
        let mut assign_stmts = Vec::new();
        let mut ascribe_stmts = Vec::new();
        let mut type_stmts = Vec::new();
        let mut trait_stmts = Vec::new();
        let mut impl_stmts = Vec::new();

        for stmt in stmts {
            match stmt {
                ast::Stmt::Ascribe(stmt) => {
                    ascribe_stmts.push(stmt);
                }

                ast::Stmt::Define(stmt) => {
                    self.register_define_stmt(&stmt);
                    assign_stmts.push(stmt);
                }

                ast::Stmt::Type(stmt) => {
                    self.register_type_stmt(&stmt);
                    type_stmts.push(stmt);
                }

                ast::Stmt::Trait(stmt) => {
                    self.register_trait_stmt(&stmt);
                    trait_stmts.push(stmt);
                }

                ast::Stmt::Impl(stmt) => {
                    impl_stmts.push(stmt);
                }

                ast::Stmt::Error(..) => {}
            }
        }

        /* add */

        for stmt in type_stmts {
            self.type_stmt(stmt);
        }

        for stmt in trait_stmts {
            self.trait_stmt(stmt);
        }

        for stmt in impl_stmts {
            self.impl_stmt(stmt);
        }

        for stmt in assign_stmts {
            self.define_stmt(stmt);
        }

        for stmt in ascribe_stmts {
            self.ascribe_stmt(stmt);
        }
    }

    fn register_type_stmt(&mut self, stmt: &ast::TypeStmt) {
        if let Some(ref ident) = stmt.var
            && let Some(var) = self.find_ty(Some(ident))
            && let Some(prev) = var.ident()
        {
            let diagnostic = Diagnostic::error("redefinition of type")
                .label(ident.span(), "found here")
                .label(prev.span(), "other definition here");

            self.reporter.report(diagnostic);

            return;
        }

        let var = Var::from(stmt.var.clone());
        self.tys.push(var.clone());

        let type_ = Type {
            var:   var.clone(),
            input: Vec::new(),
            cons:  Vec::new(),
            span:  stmt.span,
        };

        self.types.insert(var.unique(), type_);
    }

    fn register_define_stmt(&mut self, stmt: &ast::DefineStmt) {
        if let Some(ref ident) = stmt.var
            && let Some((var, _kind)) = self.find_value(Some(ident))
            && let Some(prev) = var.ident()
        {
            let diagnostic = Diagnostic::error("redefinition of variable")
                .label(ident.span(), "found here")
                .label(prev.span(), "other definition here");

            self.reporter.report(diagnostic);

            return;
        }

        let var = Var::from(stmt.var.clone());
        self.values.push((var.clone(), ValueKind::Def));

        let assignment = Def {
            ty:   None,
            var:  var.clone(),
            expr: Expr::Error(ErrorExpr { span: stmt.span }),
            span: stmt.span,
        };

        self.defs.insert(var.unique(), assignment);
    }

    fn register_trait_stmt(&mut self, stmt: &ast::TraitStmt) {
        if let Some(ref ident) = stmt.var
            && let Some(var) = self.find_kind(Some(ident))
            && let Some(prev) = var.ident()
        {
            let diagnostic = Diagnostic::error("redefinition of kind")
                .label(ident.span(), "found here")
                .label(prev.span(), "other definition here");

            self.reporter.report(diagnostic);

            return;
        }

        let var = Var::from(stmt.var.clone());
        let input = Var::from(stmt.input.clone());
        self.kinds.push(var.clone());

        let trait_ = Trait {
            var: var.clone(),
            input,
            kind: None,
            methods: Vec::new(),
            span: stmt.span,
        };

        self.traits.insert(var.unique(), trait_);
    }

    fn ascribe_stmt(&mut self, stmt: ast::AscribeStmt) {
        let ty = self.ty(stmt.ty);

        let Some((var, kind)) = self.find_value(stmt.var.as_ref()).cloned() else {
            let diagnostic = Diagnostic::error("undefined variable").label(stmt.span, "found here");
            self.reporter.report(diagnostic);
            return;
        };

        if !matches!(kind, ValueKind::Def) {
            let diagnostic = Diagnostic::error("invalid ascription").label(stmt.span, "found here");
            self.reporter.report(diagnostic);
            return;
        }

        let assignment = self.defs.get_mut(&var.unique()).unwrap();

        assignment.ty = Some(ty);
    }

    fn define_stmt(&mut self, stmt: ast::DefineStmt) {
        self.value_scope(|state| {
            let expr = state.expr(stmt.expr);

            if let Some((var, _kind)) = state.find_value(stmt.var.as_ref())
                && let Some(assignment) = state.defs.get_mut(&var.unique())
            {
                assignment.expr = expr;
            }
        });
    }

    fn type_stmt(&mut self, stmt: ast::TypeStmt) {
        self.ty_scope(|state| {
            let type_ = state.find_ty(stmt.var.as_ref()).unwrap().clone();

            let input = stmt
                .input
                .into_iter()
                .map(|input| {
                    let var = Var::from(input);
                    state.tys.push(var.clone());
                    var
                })
                .collect();

            let cons = stmt
                .cons
                .into_iter()
                .map(|cons| state.type_cons(type_.clone(), cons))
                .collect();

            let type_ = state.types.get_mut(&type_.unique()).unwrap();

            type_.input = input;
            type_.cons = cons;
        });
    }

    fn type_cons(&mut self, type_: Var, cons: ast::TypeCons) -> TypeCons {
        if let Some(ref ident) = cons.var
            && let Some(var) = self.find_cons(Some(ident))
            && let Some(prev) = var.ident()
        {
            let diagnostic = Diagnostic::error("redefinition of constructor")
                .label(ident.span(), "found here")
                .label(prev.span(), "other definition here");

            self.reporter.report(diagnostic);
        }

        if let Some(ref ident) = cons.var
            && let Some((var, _)) = self.find_value(Some(ident))
            && let Some(prev) = var.ident()
        {
            let diagnostic = Diagnostic::error("redefinition of variable")
                .label(ident.span(), "found here")
                .label(prev.span(), "other definition here");

            self.reporter.report(diagnostic);
        }

        let var = Var::from(cons.var);
        self.values.push((var.clone(), ValueKind::Cons(type_)));
        self.cons.push(var.clone());

        let args = cons.args.into_iter().map(|ty| self.ty(ty)).collect();

        TypeCons {
            var,
            args,
            span: cons.span,
        }
    }

    fn trait_stmt(&mut self, stmt: ast::TraitStmt) {
        self.ty_scope(|state| {
            let var = state.find_kind(stmt.var.as_ref()).unwrap().clone();
            let trait_ = state.traits.get(&var.unique()).unwrap();

            state.tys.push(trait_.input.clone());

            let kind = stmt.kind.map(|kind| state.kind(kind));
            let members = state.trait_members(var.clone(), stmt.members);

            let trait_ = state.traits.get_mut(&var.unique()).unwrap();

            trait_.kind = kind;
            trait_.methods = members;
        });
    }

    fn trait_members(&mut self, trait_: Var, members: Vec<ast::TraitMember>) -> Vec<TraitMethod> {
        let mut ascribe_members = Vec::new();

        for member in members {
            match member {
                ast::TraitMember::Ascribe(member) => ascribe_members.push(member),
            }
        }

        let mut members = Vec::new();

        for member in ascribe_members {
            if let Some(ref ident) = member.var
                && let Some((var, _)) = self.find_value(Some(ident))
                && let Some(prev) = var.ident()
            {
                let diagnostic = Diagnostic::error("redefinition of variable")
                    .label(ident.span(), "found here")
                    .label(prev.span(), "other definition here");

                self.reporter.report(diagnostic);
            }

            let var = Var::from(member.var);

            self.values.push((
                var.clone(),
                ValueKind::Method(trait_.clone()),
            ));

            let ty = self.ty(member.ty);

            members.push(TraitMethod {
                ty,
                var,
                span: member.span,
            });
        }

        members
    }

    fn impl_stmt(&mut self, stmt: ast::ImplStmt) {
        let Some(var) = self.find_kind(stmt.var.as_ref()).cloned() else {
            let diagnostic = Diagnostic::error("undefined trait").label(stmt.span, "here");
            self.reporter.report(diagnostic);

            return;
        };

        self.ty_scope(|state| {
            let input = state.ty(stmt.input);
            let members = state.impl_members(&var, stmt.members, stmt.span);

            let impl_ = Impl {
                var: var.clone(),
                input,
                methods: members,
                span: stmt.span,
            };

            state.impls.push(impl_);
        });
    }

    fn impl_members(
        &mut self,
        trait_: &Var,
        members: Vec<ast::ImplMember>,
        span: Span,
    ) -> Vec<ImplMethod> {
        let mut assign_members = Vec::new();

        for member in members {
            match member {
                ast::ImplMember::Assign(member) => assign_members.push(member),
            }
        }

        let mut members = Vec::new();

        for member in assign_members {
            let expr = self.expr(member.expr);

            let Some(trait_member) = self.find_trait_member(trait_, member.var.as_deref()) else {
                let diagnostic = Diagnostic::error("undefined member").label(member.span, "here");
                self.reporter.report(diagnostic);
                continue;
            };

            members.push(ImplMethod {
                var: trait_member.var.clone(),
                expr,
                span: member.span,
            });
        }

        for member in &self.traits[&trait_.unique()].methods {
            if members.iter().all(|m| m.var != member.var) {
                let diagnostic = Diagnostic::error("missing implementation")
                    .label(
                        member.span,
                        format!("of `{}`", member.var),
                    )
                    .label(span, "in impl here");

                self.reporter.report(diagnostic);
            }
        }

        members
    }
}
