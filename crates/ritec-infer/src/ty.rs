use std::fmt::{self, Debug};

use ritec_core::{Generic, Ident, Span};
use ritec_mir as mir;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TypeVariableKind {
    Integer,
    Float,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeVariable {
    pub index: usize,
    pub kind: Option<TypeVariableKind>,
}

impl TypeVariable {
    pub fn can_unify_with_var(&self, other: &Self) -> bool {
        if let (Some(kind), Some(other_kind)) = (self.kind, other.kind) {
            kind == other_kind
        } else {
            true
        }
    }

    pub fn can_unify_with_apply(&self, other: &TypeApplication) -> bool {
        match (self.kind, &other.item) {
            (Some(TypeVariableKind::Integer), ItemId::Int(_)) => true,
            (Some(TypeVariableKind::Float), ItemId::Float(_)) => true,
            (None, _) => true,
            _ => false,
        }
    }

    pub fn can_unify_with(&self, other: &InferType) -> bool {
        match other {
            InferType::Var(other) => self.can_unify_with_var(other),
            InferType::Apply(other) => self.can_unify_with_apply(other),
            _ => false,
        }
    }
}

impl Debug for TypeVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "T{}", self.index)?;

        if let Some(kind) = self.kind {
            write!(f, ": {:?}", kind)?;
        }

        Ok(())
    }
}

#[derive(Clone, PartialEq)]
pub enum ItemId {
    Void,
    Bool,
    Int(mir::IntType),
    Float(mir::FloatType),
    Pointer,
    Array(usize),
    Slice,
    Function,
    Tuple,
    Class(mir::ClassId, Ident),
    Generic(Generic),
}

impl Debug for ItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Void => write!(f, "void"),
            Self::Bool => write!(f, "bool"),
            Self::Int(ty) => write!(f, "{}", ty),
            Self::Float(ty) => write!(f, "{}", ty),
            Self::Pointer => write!(f, "*"),
            Self::Array(size) => write!(f, "[{}]", size),
            Self::Slice => write!(f, "[]"),
            Self::Function => write!(f, "fn"),
            Self::Tuple => write!(f, "()"),
            Self::Class(_, ident) => write!(f, "{}", ident),
            Self::Generic(generic) => write!(f, "{}", generic),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct TypeApplication {
    pub item: ItemId,
    pub arguments: Vec<InferType>,
    pub span: Span,
}

impl Debug for TypeApplication {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let arguments = self
            .arguments
            .iter()
            .map(|ty| format!("{:?}", ty))
            .collect::<Vec<_>>()
            .join(", ");

        write!(f, "{:?}<{}>", self.item, arguments)
    }
}

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct TypeProjection {}

#[derive(Clone, PartialEq)]
pub enum InferType {
    Var(TypeVariable),
    Apply(TypeApplication),
    Proj(TypeProjection),
}

impl InferType {
    pub const fn void(span: Span) -> Self {
        Self::Apply(TypeApplication {
            item: ItemId::Void,
            arguments: Vec::new(),
            span,
        })
    }

    pub fn apply(item: ItemId, arguments: impl Into<Vec<InferType>>, span: Span) -> Self {
        let apply = TypeApplication {
            item,
            arguments: arguments.into(),
            span,
        };

        Self::Apply(apply)
    }
}

impl From<TypeVariable> for InferType {
    fn from(value: TypeVariable) -> Self {
        Self::Var(value)
    }
}

impl From<TypeApplication> for InferType {
    fn from(value: TypeApplication) -> Self {
        Self::Apply(value)
    }
}

impl From<TypeProjection> for InferType {
    fn from(value: TypeProjection) -> Self {
        Self::Proj(value)
    }
}

impl Debug for InferType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Var(var) => write!(f, "{:?}", var),
            Self::Apply(apply) => write!(f, "{:?}", apply),
            Self::Proj(proj) => write!(f, "{:?}", proj),
        }
    }
}
