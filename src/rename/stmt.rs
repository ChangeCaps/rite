use std::{collections::HashSet, mem};

use crate::{
    ast,
    diagnostic::{Diagnostic, Span},
    rename::{
        self, ErrorExpr, ErrorTy, Expr, Kind, Scope, ScopeCons, ScopeTy, ScopeValue,
        ScopeValueKind, State, Ty, TyPrim, Vis,
    },
    var::{Ident, Var},
};

use super::ScopeTyKind;

pub struct Def {
    pub ty:   Option<Ty>,
    pub var:  Var,
    pub expr: Expr,
    pub span: Span,
}

pub struct Type {
    pub var:   Var,
    pub input: Vec<Var>,
    pub cons:  TypeCons,
    pub span:  Span,
}

pub enum TypeCons {
    Rec(RecordCons),
    Tuple(Vec<TupleCons>),
}

pub struct RecordCons {
    pub var:    Var,
    pub fields: Vec<RecordField>,
    pub span:   Span,
}

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
    pub fn modules(&mut self, modules: Vec<ast::Module>) {
        let intrinsic = self.intrinsic();
        self.modules.insert(String::from("intrinsic"), intrinsic);

        let imports = modules
            .iter()
            .map(|module| module.path.clone())
            .chain([String::from("intrinsic")])
            .collect::<HashSet<_>>();

        let mut scope = Scope::new();

        scope.imports.extend(
            rename::PRELUDE
                .iter()
                .copied()
                .filter(|path| imports.contains(*path))
                .map(Into::into),
        );

        let mut define_stmts = Vec::new();
        let mut ascribe_stmts = Vec::new();
        let mut type_stmts = Vec::new();
        let mut trait_stmts = Vec::new();
        let mut impl_stmts = Vec::new();

        for module in modules {
            let mut scope = scope.clone();

            if module.path == "std:math" {
                scope.tys.push(ScopeTy {
                    name: Some(String::from("int")),
                    kind: ScopeTyKind::Prim(TyPrim::Int),
                });

                scope.tys.push(ScopeTy {
                    name: Some(String::from("real")),
                    kind: ScopeTyKind::Prim(TyPrim::Real),
                });
            }

            if module.path == "std:str" {
                scope.tys.push(ScopeTy {
                    name: Some(String::from("str")),
                    kind: ScopeTyKind::Prim(TyPrim::Str),
                });
            }

            if module.path == "std:system" {
                scope.tys.push(ScopeTy {
                    name: Some(String::from("io")),
                    kind: ScopeTyKind::Prim(TyPrim::Io),
                });
            }

            for stmt in module.stmts {
                match stmt {
                    ast::Stmt::Import(stmt) => {
                        self.register_import_stmt(&imports, &mut scope, stmt);
                    }

                    ast::Stmt::Ascribe(stmt) => {
                        ascribe_stmts.push((module.path.clone(), stmt));
                    }

                    ast::Stmt::Define(stmt) => {
                        let var = self.register_define_stmt(&mut scope, &stmt);
                        define_stmts.push((module.path.clone(), var, stmt));
                    }

                    ast::Stmt::Type(stmt) => {
                        let var = self.register_type_stmt(&mut scope, &stmt);
                        type_stmts.push((module.path.clone(), var, stmt));
                    }

                    ast::Stmt::Trait(stmt) => {
                        let var = self.register_trait_stmt(&mut scope, &stmt);
                        trait_stmts.push((module.path.clone(), var, stmt));
                    }

                    ast::Stmt::Impl(stmt) => {
                        impl_stmts.push((module.path.clone(), stmt));
                    }

                    ast::Stmt::Error(..) => {}
                }
            }

            self.modules.insert(module.path, scope);
        }

        for (module, var, stmt) in type_stmts {
            let scope = self.modules[&module].clone();
            self.type_stmt(scope, var, stmt);
        }

        for (module, var, stmt) in trait_stmts {
            let scope = self.modules[&module].clone();
            self.trait_stmt(scope, var, stmt);
        }

        for (module, stmt) in ascribe_stmts {
            let scope = self.modules[&module].clone();
            self.ascribe_stmt(scope, stmt);
        }

        for (module, var, stmt) in define_stmts {
            let scope = self.modules[&module].clone();
            self.define_stmt(scope, var, stmt);
        }

        for (module, stmt) in impl_stmts {
            let scope = self.modules[&module].clone();
            self.impl_stmt(scope, stmt);
        }
    }

    fn register_import_stmt(
        &mut self,
        imports: &HashSet<String>,
        scope: &mut Scope,
        stmt: ast::ImportStmt,
    ) {
        let Some(path) = stmt.path else { return };

        if !imports.contains(&path) {
            let diagnostic =
                Diagnostic::error("invalid import path").label(stmt.span, "in import here");

            self.reporter.report(diagnostic);

            return;
        }
        scope.imports.push(path.clone());

        let Some(alias) = stmt.alias else { return };

        if let Some(prev) = scope.find_value(Some(&alias))
            && let Some(prev) = prev.var.ident()
        {
            let diagnostic = Diagnostic::error("redefinition of import")
                .label(alias.span(), "found here")
                .label(prev.span(), "other definition here");

            self.reporter.report(diagnostic);
        }

        let value = ScopeValue {
            vis:  Vis::Local,
            var:  Var::new(alias),
            kind: ScopeValueKind::Import(path.clone()),
        };

        scope.values.push(value);
    }

    fn register_type_stmt(&mut self, scope: &mut Scope, stmt: &ast::TypeStmt) -> Var {
        if let Some(ref ident) = stmt.var
            && let Some(prev) = scope.find_ty(Some(ident))
            && let Some(prev) = prev.ident()
        {
            let diagnostic = Diagnostic::error("redefinition of type")
                .label(ident.span(), "found here")
                .label(prev.span(), "other definition here");

            self.reporter.report(diagnostic);
        }

        let var = Var::from(stmt.var.clone());

        scope.tys.push(ScopeTy {
            name: var.ident().map(|ident| ident.as_str().into()),
            kind: ScopeTyKind::Type(var.clone()),
        });

        let input = stmt.input.iter().cloned().map(Var::from).collect();

        let cons = match stmt.cons {
            ast::TypeCons::Record(ref cons) => {
                TypeCons::Rec(self.register_record_cons(scope, var.clone(), cons))
            }

            ast::TypeCons::Tuple(ref cons) => TypeCons::Tuple(
                cons.iter()
                    .map(|cons| self.register_tuple_cons(scope, var.clone(), cons))
                    .collect(),
            ),
        };

        let type_ = Type {
            var: var.clone(),
            input,
            cons,
            span: stmt.span,
        };

        self.types.insert(var.unique(), type_);
        var
    }

    fn register_record_cons(
        &mut self,
        scope: &mut Scope,
        type_: Var,
        cons: &ast::RecordCons,
    ) -> RecordCons {
        let var = Var::from(cons.var.clone());

        let value = ScopeValue {
            vis:  Vis::Public,
            var:  var.clone(),
            kind: ScopeValueKind::Cons(type_),
        };

        scope.values.push(value);

        RecordCons {
            var,
            fields: Vec::new(),
            span: cons.span,
        }
    }

    fn register_tuple_cons(
        &mut self,
        scope: &mut Scope,
        type_: Var,
        ast: &ast::TupleCons,
    ) -> TupleCons {
        if let Some(ref ident) = ast.var
            && let Some(cons) = scope.find_cons(Some(ident))
            && let Some(prev) = cons.var.ident()
        {
            let diagnostic = Diagnostic::error("redefinition of constructor")
                .label(ident.span(), "found here")
                .label(prev.span(), "other definition here");

            self.reporter.report(diagnostic);
        }

        if let Some(ref ident) = ast.var
            && let Some(value) = scope.find_value(Some(ident))
            && let Some(prev) = value.var.ident()
        {
            let diagnostic = Diagnostic::error("redefinition of variable")
                .label(ident.span(), "found here")
                .label(prev.span(), "other definition here");

            self.reporter.report(diagnostic);
        }

        let var = Var::from(ast.var.clone());

        let value = ScopeValue {
            vis:  Vis::Public,
            var:  var.clone(),
            kind: ScopeValueKind::Cons(type_.clone()),
        };

        let cons = ScopeCons {
            var:   var.clone(),
            type_: type_.clone(),
        };

        scope.values.push(value);
        scope.cons.push(cons);

        TupleCons {
            var,
            args: Vec::new(),
            span: ast.span,
        }
    }

    fn register_trait_stmt(&mut self, scope: &mut Scope, stmt: &ast::TraitStmt) -> Var {
        if let Some(ref ident) = stmt.var
            && let Some(var) = scope.find_kind(Some(ident))
            && let Some(prev) = var.ident()
        {
            let diagnostic = Diagnostic::error("redefinition of kind")
                .label(ident.span(), "found here")
                .label(prev.span(), "other definition here");

            self.reporter.report(diagnostic);
        }

        let var = Var::from(stmt.var.clone());
        let input = Var::from(stmt.input.clone());
        scope.kinds.push(var.clone());

        let mut methods = Vec::new();

        for member in &stmt.members {
            match member {
                ast::TraitMember::Ascribe(member) => {
                    let method = self.register_trait_method(scope, var.clone(), member);
                    methods.push(method);
                }
            }
        }

        let trait_ = Trait {
            var: var.clone(),
            input,
            kind: None,
            methods,
            span: stmt.span,
        };

        self.traits.insert(var.unique(), trait_);

        var
    }

    fn register_trait_method(
        &mut self,
        scope: &mut Scope,
        trait_: Var,
        member: &ast::AscribeStmt,
    ) -> TraitMethod {
        if let Some(ref ident) = member.var
            && let Some(value) = scope.find_value(Some(ident))
            && let Some(prev) = value.var.ident()
        {
            let diagnostic = Diagnostic::error("redefinition of method")
                .label(ident.span(), "found here")
                .label(prev.span(), "other definition here");

            self.reporter.report(diagnostic);
        }

        let var = Var::from(member.var.clone());

        let value = ScopeValue {
            vis:  Vis::Public,
            var:  var.clone(),
            kind: ScopeValueKind::Method(trait_),
        };

        scope.values.push(value);

        TraitMethod {
            ty: Ty::Error(ErrorTy { span: member.span }),
            var,
            span: member.span,
        }
    }

    fn register_define_stmt(&mut self, scope: &mut Scope, stmt: &ast::DefineStmt) -> Var {
        if let Some(ref ident) = stmt.var
            && let Some(value) = scope.find_value(Some(ident))
            && let Some(prev) = value.var.ident()
        {
            let diagnostic = Diagnostic::error("redefinition of variable")
                .label(ident.span(), "found here")
                .label(prev.span(), "other definition here");

            self.reporter.report(diagnostic);
        }

        let var = Var::from(stmt.var.clone());

        let value = ScopeValue {
            vis:  Vis::Public,
            var:  var.clone(),
            kind: ScopeValueKind::Def,
        };

        scope.values.push(value);

        let assignment = Def {
            ty:   None,
            var:  var.clone(),
            expr: Expr::Error(ErrorExpr { span: stmt.span }),
            span: stmt.span,
        };

        self.defs.insert(var.unique(), assignment);

        var
    }

    fn ascribe_stmt(&mut self, mut scope: Scope, stmt: ast::AscribeStmt) {
        let ty = self.ty(&mut scope, stmt.ty);

        let Some(value) = scope.find_value(stmt.var.as_deref()).cloned() else {
            let diagnostic = Diagnostic::error("undefined variable").label(stmt.span, "found here");
            self.reporter.report(diagnostic);
            return;
        };

        if !matches!(value.kind, ScopeValueKind::Def) {
            let diagnostic = Diagnostic::error("invalid ascription").label(stmt.span, "found here");
            self.reporter.report(diagnostic);
            return;
        }

        let def = self.defs.get_mut(&value.var.unique()).unwrap();
        def.ty = Some(ty);
    }

    fn define_stmt(&mut self, scope: Scope, var: Var, stmt: ast::DefineStmt) {
        let expr = self.expr_with_args(scope, stmt.args, stmt.expr);

        if let Some(def) = self.defs.get_mut(&var.unique()) {
            def.expr = expr;
        }
    }

    fn type_stmt(&mut self, mut scope: Scope, var: Var, stmt: ast::TypeStmt) {
        let type_ = self.type_mut(&var);
        scope
            .tys
            .extend(type_.input.iter().cloned().map(From::from));

        match stmt.cons {
            ast::TypeCons::Record(cons) => {
                let fields = cons
                    .fields
                    .into_iter()
                    .filter_map(|field| {
                        Some(RecordField {
                            name: field.name?,
                            ty:   self.ty(&mut scope, field.ty),
                            span: field.span,
                        })
                    })
                    .collect::<Vec<_>>();

                let TypeCons::Rec(ref mut cons) = self.type_mut(&var).cons else {
                    panic!();
                };

                cons.fields = fields;
            }

            ast::TypeCons::Tuple(cons) => {
                let args = cons
                    .into_iter()
                    .map(|cons| {
                        cons.args
                            .into_iter()
                            .map(|arg| self.ty(&mut scope, arg))
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();

                let TypeCons::Tuple(ref mut cons) = self.type_mut(&var).cons else {
                    panic!();
                };

                for (cons, args) in cons.iter_mut().zip(args) {
                    cons.args = args;
                }
            }
        }
    }

    fn trait_stmt(&mut self, mut scope: Scope, var: Var, stmt: ast::TraitStmt) {
        let trait_ = self.trait_mut(&var);
        scope.tys.push(trait_.input.clone().into());

        let mut methods = mem::take(&mut trait_.methods);

        for member in stmt.members {
            match member {
                ast::TraitMember::Ascribe(member) => {
                    let method = methods
                        .iter_mut()
                        .find(|method| method.var.ident() == member.var.as_ref())
                        .unwrap();

                    let mut scope = scope.clone();
                    method.ty = self.ty(&mut scope, member.ty);
                }
            }
        }

        let kind = stmt.kind.map(|kind| self.kind(&scope, kind));

        let trait_ = self.trait_mut(&var);

        trait_.methods = methods;
        trait_.kind = kind;
    }

    fn impl_stmt(&mut self, mut scope: Scope, stmt: ast::ImplStmt) {
        let Some(var) = self.find_kind(&scope, stmt.var.as_ref()).cloned() else {
            let diagnostic = Diagnostic::error("undefined trait").label(stmt.span, "here");
            self.reporter.report(diagnostic);

            return;
        };

        let input = self.ty(&mut scope, stmt.input);
        let members = self.impl_members(&scope, &var, stmt.members, stmt.span);

        let impl_ = Impl {
            var: var.clone(),
            input,
            methods: members,
            span: stmt.span,
        };

        self.impls.push(impl_);
    }

    fn impl_members(
        &mut self,
        scope: &Scope,
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
            let expr = self.expr_with_args(scope.clone(), member.args, member.expr);

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
