use ritec_core::{Id, Span};
use ritec_mir::{LocalId, Type};

pub type ExprId = Id<Expr>;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Local(LocalExpr),
    Ref(RefExpr),
    Deref(DerefExpr),
    Assign(AssignExpr),
    Return(ReturnExpr),
}

impl Expr {
    pub fn ty(&self) -> &Type {
        match self {
            Expr::Local(local) => &local.ty,
            Expr::Ref(expr) => &expr.ty,
            Expr::Deref(expr) => &expr.ty,
            Expr::Assign(expr) => &expr.ty,
            Expr::Return(expr) => &expr.ty,
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Expr::Local(expr) => expr.span,
            Expr::Ref(expr) => expr.span,
            Expr::Deref(expr) => expr.span,
            Expr::Assign(expr) => expr.span,
            Expr::Return(expr) => expr.span,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalExpr {
    pub local: LocalId,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RefExpr {
    pub operand: ExprId,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DerefExpr {
    pub operand: ExprId,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AssignExpr {
    pub lhs: ExprId,
    pub rhs: ExprId,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReturnExpr {
    pub value: Option<ExprId>,
    pub ty: Type,
    pub span: Span,
}
