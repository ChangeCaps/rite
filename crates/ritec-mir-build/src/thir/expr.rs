use ritec_core::{BinOp, Id, Literal, Span, UnaryOp};
use ritec_hir::FunctionId;
use ritec_mir::{ClassType, FieldId, LocalId, Type};

use super::BlockId;

pub type ExprId = Id<Expr>;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Local(LocalExpr),
    Literal(LiteralExpr),
    Function(FunctionExpr),
    Init(InitExpr),
    Field(FieldExpr),
    Bitcast(BitcastExpr),
    Call(CallExpr),
    StaticCall(StaticCallExpr),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Assign(AssignExpr),
    Return(ReturnExpr),
    Break(BreakExpr),
    Block(BlockExpr),
    If(IfExpr),
    Loop(LoopExpr),
}

impl Expr {
    pub fn ty(&self) -> &Type {
        match self {
            Expr::Local(expr) => &expr.ty,
            Expr::Literal(expr) => &expr.ty,
            Expr::Function(expr) => &expr.ty,
            Expr::Init(expr) => &expr.ty,
            Expr::Field(expr) => &expr.ty,
            Expr::Bitcast(expr) => &expr.ty,
            Expr::Call(expr) => &expr.ty,
            Expr::StaticCall(expr) => &expr.ty,
            Expr::Unary(expr) => &expr.ty,
            Expr::Binary(expr) => &expr.ty,
            Expr::Assign(expr) => &expr.ty,
            Expr::Return(expr) => &expr.ty,
            Expr::Break(expr) => &expr.ty,
            Expr::Block(expr) => &expr.ty,
            Expr::If(expr) => &expr.ty,
            Expr::Loop(expr) => &expr.ty,
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Expr::Local(expr) => expr.span,
            Expr::Literal(expr) => expr.span,
            Expr::Function(expr) => expr.span,
            Expr::Init(expr) => expr.span,
            Expr::Field(expr) => expr.span,
            Expr::Bitcast(expr) => expr.span,
            Expr::Call(expr) => expr.span,
            Expr::StaticCall(expr) => expr.span,
            Expr::Unary(expr) => expr.span,
            Expr::Binary(expr) => expr.span,
            Expr::Assign(expr) => expr.span,
            Expr::Return(expr) => expr.span,
            Expr::Break(expr) => expr.span,
            Expr::Block(expr) => expr.span,
            Expr::If(expr) => expr.span,
            Expr::Loop(expr) => expr.span,
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
pub struct LiteralExpr {
    pub literal: Literal,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionExpr {
    pub function: FunctionId,
    pub generics: Vec<Type>,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InitExpr {
    pub class: ClassType,
    pub fields: Vec<(FieldId, ExprId)>,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FieldExpr {
    pub class: ExprId,
    pub field: FieldId,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BitcastExpr {
    pub expr: ExprId,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CallExpr {
    pub callee: ExprId,
    pub arguments: Vec<ExprId>,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StaticCallExpr {
    pub callee: FunctionId,
    pub generics: Vec<Type>,
    pub arguments: Vec<ExprId>,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnaryExpr {
    pub operator: UnaryOp,
    pub operand: ExprId,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryExpr {
    pub operator: BinOp,
    pub lhs: ExprId,
    pub rhs: ExprId,
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

#[derive(Clone, Debug, PartialEq)]
pub struct BreakExpr {
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BlockExpr {
    pub block: BlockId,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfExpr {
    pub condition: ExprId,
    pub then_expr: ExprId,
    pub else_expr: Option<ExprId>,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoopExpr {
    pub block: BlockId,
    pub ty: Type,
    pub span: Span,
}
