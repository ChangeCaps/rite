use ritec_core::{BinOp, Id, Ident, Literal, Span, UnaryOp};

use crate::{BlockId, ClassType, FieldId, FunctionInstance, HirId, LocalId, Type};

pub type ExprId = Id<Expr>;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Local(LocalExpr),
    Literal(LiteralExpr),
    Function(FunctionExpr),
    ClassInit(ClassInitExpr),
    Field(FieldExpr),
    As(AsExpr),
    Bitcast(BitcastExpr),
    Sizeof(SizeofExpr),
    Alignof(AlignofExpr),
    Malloc(MallocExpr),
    Free(FreeExpr),
    Memcpy(MemcpyExpr),
    Call(CallExpr),
    MethodCall(MethodCallExpr),
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
            Expr::ClassInit(expr) => expr.span,
            Expr::Field(expr) => expr.span,
            Expr::As(expr) => expr.span,
            Expr::Bitcast(expr) => expr.span,
            Expr::Sizeof(expr) => expr.span,
            Expr::Alignof(expr) => expr.span,
            Expr::Malloc(expr) => expr.span,
            Expr::Free(expr) => expr.span,
            Expr::Memcpy(expr) => expr.span,
            Expr::Call(expr) => expr.span,
            Expr::MethodCall(expr) => expr.span,
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
            Expr::ClassInit(expr) => expr.id,
            Expr::Field(expr) => expr.id,
            Expr::As(expr) => expr.id,
            Expr::Bitcast(expr) => expr.id,
            Expr::Sizeof(expr) => expr.id,
            Expr::Alignof(expr) => expr.id,
            Expr::Malloc(expr) => expr.id,
            Expr::Free(expr) => expr.id,
            Expr::Memcpy(expr) => expr.id,
            Expr::Call(expr) => expr.id,
            Expr::MethodCall(expr) => expr.id,
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

impl From<ClassInitExpr> for Expr {
    fn from(expr: ClassInitExpr) -> Self {
        Self::ClassInit(expr)
    }
}

impl From<FieldExpr> for Expr {
    fn from(expr: FieldExpr) -> Self {
        Self::Field(expr)
    }
}

impl From<AsExpr> for Expr {
    fn from(expr: AsExpr) -> Self {
        Self::As(expr)
    }
}

impl From<BitcastExpr> for Expr {
    fn from(expr: BitcastExpr) -> Self {
        Self::Bitcast(expr)
    }
}

impl From<SizeofExpr> for Expr {
    fn from(expr: SizeofExpr) -> Self {
        Self::Sizeof(expr)
    }
}

impl From<AlignofExpr> for Expr {
    fn from(expr: AlignofExpr) -> Self {
        Self::Alignof(expr)
    }
}

impl From<MallocExpr> for Expr {
    fn from(expr: MallocExpr) -> Self {
        Self::Malloc(expr)
    }
}

impl From<FreeExpr> for Expr {
    fn from(expr: FreeExpr) -> Self {
        Self::Free(expr)
    }
}

impl From<MemcpyExpr> for Expr {
    fn from(expr: MemcpyExpr) -> Self {
        Self::Memcpy(expr)
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
pub struct ClassInitExpr {
    pub class: ClassType,
    pub fields: Vec<(FieldId, ExprId)>,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FieldExpr {
    pub class: ExprId,
    pub field: Ident,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AsExpr {
    pub expr: ExprId,
    pub ty: Type,
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
pub struct SizeofExpr {
    pub ty: Type,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AlignofExpr {
    pub ty: Type,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MallocExpr {
    pub ty: Type,
    pub count: ExprId,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FreeExpr {
    pub expr: ExprId,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemcpyExpr {
    pub dst: ExprId,
    pub src: ExprId,
    pub size: ExprId,
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
pub struct MethodCallExpr {
    pub callee: ExprId,
    pub method: Ident,
    pub generics: Vec<Type>,
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
    pub operator: BinOp,
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
    pub then_expr: ExprId,
    pub else_expr: Option<ExprId>,
    pub id: HirId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoopExpr {
    pub block: BlockId,
    pub id: HirId,
    pub span: Span,
}
