#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ItemId {
    Var(VarId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VarId {
    data: u32,
}
