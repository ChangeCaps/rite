use crate::{
    ast::{Expr, Kind, Pat, Ty},
    diagnostic::Span,
    var::Ident,
};

pub enum Stmt {
    Import(ImportStmt),
    Define(DefineStmt),
    Ascribe(AscribeStmt),
    Type(TypeStmt),
    Trait(TraitStmt),
    Impl(ImplStmt),
    Error(ErrorStmt),
}

pub struct ImportStmt {
    pub path:  Option<String>,
    pub alias: Option<Ident>,
    pub span:  Span,
}

pub struct DefineStmt {
    pub var:  Option<Ident>,
    pub args: Vec<Pat>,
    pub expr: Expr,
    pub span: Span,
}

pub struct AscribeStmt {
    pub var:  Option<Ident>,
    pub ty:   Ty,
    pub span: Span,
}

pub struct TypeStmt {
    pub var:   Option<Ident>,
    pub input: Vec<Option<Ident>>,
    pub cons:  TypeCons,
    pub span:  Span,
}

pub enum TypeCons {
    Record(RecordCons),
    Tuple(Vec<TupleCons>),
}

pub struct RecordCons {
    pub var:    Option<Ident>,
    pub fields: Vec<RecordField>,
    pub span:   Span,
}

pub struct RecordField {
    pub name: Option<Ident>,
    pub ty:   Ty,
    pub span: Span,
}

pub struct TupleCons {
    pub var:  Option<Ident>,
    pub args: Vec<Ty>,
    pub span: Span,
}

pub struct TraitStmt {
    pub var:     Option<Ident>,
    pub input:   Option<Ident>,
    pub kind:    Option<Kind>,
    pub members: Vec<TraitMember>,
    pub span:    Span,
}

pub enum TraitMember {
    Ascribe(AscribeStmt),
}

pub struct ImplStmt {
    pub var:     Option<Ident>,
    pub input:   Ty,
    pub members: Vec<ImplMember>,
    pub span:    Span,
}

pub enum ImplMember {
    Assign(DefineStmt),
}

pub struct ErrorStmt {
    pub span: Span,
}
