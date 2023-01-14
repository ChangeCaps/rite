use ritec_infer::Error as InferError;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Infer(InferError),
}

impl From<InferError> for Error {
    fn from(err: InferError) -> Self {
        Self::Infer(err)
    }
}
