#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum BinPrim {
    Add(PrimNum),
    Sub(PrimNum),
    Mul(PrimNum),
    Div(PrimNum),
}

#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum PrimNum {
    Int,
    Real,
}
