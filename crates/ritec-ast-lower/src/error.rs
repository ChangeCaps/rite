#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    ModuleRegistration,
    FunctionRegistration,
    FunctionCompletion,
}
