#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum UnPrim {
    IoPure,
    StrLen,
    Str(PrimNum),
}

#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum BinPrim {
    Trace,
    IoBind,
    StrConcat,
    StrTake,
    StrSkip,
    StrChar,
    StrFind,
    StrCmp,
    Add(PrimNum),
    Sub(PrimNum),
    Mul(PrimNum),
    Div(PrimNum),
    Gt(PrimNum),
    Lt(PrimNum),
}

#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum PrimNum {
    Int,
    Real,
}
