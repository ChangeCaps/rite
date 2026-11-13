#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum UnPrim {
    IoPure,
}

#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum BinPrim {
    IoBind,
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
