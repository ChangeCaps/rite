use crate::{
    ast::Pat,
    diagnostic::Span,
    var::{Ident, Operator},
};

pub enum Expr {
    Lit(LitExpr),
    Let(LetExpr),
    Var(VarExpr),
    Lam(LamExpr),
    App(AppExpr),
    Bin(BinExpr),
    Seq(SeqExpr),
    Rec(RecExpr),
    List(ListExpr),
    Paren(ParenExpr),
    Field(FieldExpr),
    Match(MatchExpr),
    Error(ErrorExpr),
}

pub struct LitExpr {
    pub lit:  Lit,
    pub span: Span,
}

pub enum Lit {
    Str(String),
    Int(i64),
    Real(f64),
}

pub struct LetExpr {
    pub pat:    Pat,
    pub assign: Assign,
    pub input:  Box<Expr>,
    pub output: Box<Expr>,
    pub span:   Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Assign {
    Assign,
    Bind,
}

pub struct VarExpr {
    pub var:  Option<Ident>,
    pub span: Span,
}

pub struct LamExpr {
    pub args:   Vec<Pat>,
    pub output: Box<Expr>,
    pub span:   Span,
}

pub struct AppExpr {
    pub lambda: Box<Expr>,
    pub input:  Box<Expr>,
    pub span:   Span,
}

pub struct BinExpr {
    pub op:   Operator,
    pub lhs:  Box<Expr>,
    pub rhs:  Box<Expr>,
    pub span: Span,
}

pub struct SeqExpr {
    pub input:  Box<Expr>,
    pub output: Box<Expr>,
    pub span:   Span,
}

pub struct RecExpr {
    pub input:  Box<Expr>,
    pub fields: Vec<ExprField>,
    pub span:   Span,
}

pub struct ExprField {
    pub name: Option<Ident>,
    pub expr: Expr,
    pub span: Span,
}

pub struct ListExpr {
    pub items: Vec<Expr>,
    pub span:  Span,
}

pub struct ParenExpr {
    pub expr: Box<Expr>,
    pub span: Span,
}

pub struct FieldExpr {
    pub input: Box<Expr>,
    pub name:  Option<Ident>,
    pub span:  Span,
}

pub struct MatchExpr {
    pub input: Box<Expr>,
    pub arms:  Vec<MatchArm>,
    pub span:  Span,
}

pub struct MatchArm {
    pub pat:    Pat,
    pub output: Expr,
    pub span:   Span,
}

pub struct ErrorExpr {
    pub span: Span,
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Self::Lit(expr) => expr.span,
            Self::Let(expr) => expr.span,
            Self::Var(expr) => expr.span,
            Self::Lam(expr) => expr.span,
            Self::App(expr) => expr.span,
            Self::Bin(expr) => expr.span,
            Self::Seq(expr) => expr.span,
            Self::Rec(expr) => expr.span,
            Self::List(expr) => expr.span,
            Self::Paren(expr) => expr.span,
            Self::Field(expr) => expr.span,
            Self::Match(expr) => expr.span,
            Self::Error(expr) => expr.span,
        }
    }
}
