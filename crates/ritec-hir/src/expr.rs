use ritec_core::{BinaryOp, Id, Literal, Span, UnaryOp};

use crate::{BlockId, FunctionInstance, HirId, LocalId, Type};

pub type ExprId = Id<Expr>;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Local(LocalExpr),
    Literal(LiteralExpr),
    Function(FunctionExpr),
    Bitcast(BitcastExpr),
    Call(CallExpr),
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
    pub const fn span(&self) -> Span {
        match self {
            Expr::Local(expr) => expr.span,
            Expr::Literal(expr) => expr.span,
            Expr::Function(expr) => expr.span,
            Expr::Bitcast(expr) => expr.span,
            Expr::Call(expr) => expr.span,
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

    pub const fn id(&self) -> HirId {
        match self {
            Expr::Local(expr) => expr.id,
            Expr::Literal(expr) => expr.id,
            Expr::Function(expr) => expr.id,
            Expr::Bitcast(expr) => expr.id,
            Expr::Call(expr) => expr.id,
            Expr::Unary(expr) => expr.id,
            Expr::Binary(expr) => expr.id,
            Expr::Assign(expr) => expr.id,
            Expr::Return(expr) => expr.id,
            Expr::Break(expr) => expr.id,
            Expr::Block(expr) => expr.id,
            Expr::If(expr) => expr.id,
            Expr::Loop(expr) => expr.id,
        }
    }
}

impl From<LocalExpr> for Expr {
    fn from(expr: LocalExpr) -> Self {
        Self::Local(expr)
    }
}

impl From<LiteralExpr> for Expr {
    fn from(expr: LiteralExpr) -> Self {
        Self::Literal(expr)
    }
}

impl From<FunctionExpr> for Expr {
    fn from(expr: FunctionExpr) -> Self {
        Self::Function(expr)
    }
}

impl From<BitcastExpr> for Expr {
    fn from(expr: BitcastExpr) -> Self {
        Self::Bitcast(expr)
    }
}

impl From<CallExpr> for Expr {
    fn from(expr: CallExpr) -> Self {
        Self::Call(expr)
    }
}

impl From<UnaryExpr> for Expr {
    fn from(expr: UnaryExpr) -> Self {
        Self::Unary(expr)
    }
}

impl From<BinaryExpr> for Expr {
    fn from(expr: BinaryExpr) -> Self {
        Self::Binary(expr)
    }
}

impl From<AssignExpr> for Expr {
    fn from(expr: AssignExpr) -> Self {
        Self::Assign(expr)
    }
}

impl From<ReturnExpr> for Expr {
    fn from(expr: ReturnExpr) -> Self {
        Self::Return(expr)
    }
}

impl From<BreakExpr> for Expr {
    fn from(expr: BreakExpr) -> Self {
        Self::Break(expr)
    }
}

impl From<BlockExpr> for Expr {
    fn from(expr: BlockExpr) -> Self {
        Self::Block(expr)
    }
}

impl From<IfExpr> for Expr {
    fn from(expr: IfExpr) -> Self {
        Self::If(expr)
    }
}

impl From<LoopExpr> for Expr {
    fn from(expr: LoopExpr) -> Self {
        Self::Loop(expr)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalExpr {
    pub local: LocalId,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LiteralExpr {
    pub literal: Literal,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionExpr {
    pub instance: FunctionInstance,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BitcastExpr {
    pub expr: ExprId,
    pub ty: Type,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CallExpr {
    pub callee: ExprId,
    pub arguments: Vec<ExprId>,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnaryExpr {
    pub operator: UnaryOp,
    pub operand: ExprId,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryExpr {
    pub operator: BinaryOp,
    pub lhs: ExprId,
    pub rhs: ExprId,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AssignExpr {
    pub lhs: ExprId,
    pub rhs: ExprId,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReturnExpr {
    pub value: Option<ExprId>,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BreakExpr {
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BlockExpr {
    pub block: BlockId,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfExpr {
    pub condition: ExprId,
    pub then_block: BlockId,
    pub else_block: Option<ExprId>,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoopExpr {
    pub block: BlockId,
    pub id: HirId,
    pub span: Span,
}
