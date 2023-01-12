use ritec_ast::Path;
use ritec_infer::InferError;

#[derive(Clone, Debug, PartialEq)]
pub enum LowerError {
    /// The type is inferred in a place where type inference is not allowed.
    /// For example in a function argument.
    InvalidInferred,
    /// No type was defined for the given path.
    UndefinedType(Path),
    InvalidPath(Path),
    Infer(InferError),
}

impl From<InferError> for LowerError {
    fn from(err: InferError) -> Self {
        Self::Infer(err)
    }
}
