#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    ModuleRegistration,
    ClassRegistration,
    ClassCompletion,
    FunctionRegistration,
    FunctionCompletion,
}
