use std::{
    fmt::{self, Display},
    ops::Deref,
};

use ritec_core::Span;

use crate::{ParseStream, Peek};

macro_rules! keyword {
    { $($str:literal => $ident:ident),* $(,)? } => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum KeywordKind {
            $($ident),*
        }

        impl KeywordKind {
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$ident => $str),*
                }
            }

            pub fn from_str(s: &str) -> Option<Self> {
                match s {
                    $($str => Some(Self::$ident)),*,
                    _ => None,
                }
            }
        }
    };
}

keyword! {
    /* types */
    "void" => Void,

    /* boolean */
    "bool" => Bool,
    "true" => True,
    "false" => False,

    /* signed integer */
    "i8" => I8,
    "i16" => I16,
    "i32" => I32,
    "i64" => I64,
    "i128" => I128,
    "isize" => Isize,

    /* unsigned integer */
    "u8" => U8,
    "u16" => U16,
    "u32" => U32,
    "u64" => U64,
    "u128" => U128,
    "usize" => Usize,

    /* floating point */
    "f16" => F16,
    "f32" => F32,
    "f64" => F64,

    "self" => SelfLower,
    "super" => Super,
    "let" => Let,
    "fn" => Fn,
    "if" => If,
    "else" => Else,
    "return" => Return,
    "break" => Break,
    "while" => While,
    "loop" => Loop,
    "mod" => Mod,
}

impl Display for KeywordKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Peek for KeywordKind {
    fn peek(&self, parser: ParseStream) -> bool {
        if let Some(keyword) = parser.peek_keyword() {
            keyword.kind == *self
        } else {
            false
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Keyword {
    kind: KeywordKind,
    span: Span,
}

impl Keyword {
    pub const fn new(kind: KeywordKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub const fn kind(&self) -> KeywordKind {
        self.kind
    }

    pub const fn span(&self) -> Span {
        self.span
    }
}

impl Deref for Keyword {
    type Target = KeywordKind;

    fn deref(&self) -> &Self::Target {
        &self.kind
    }
}

impl Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str() {
        assert_eq!(KeywordKind::Let.to_string(), "let");
    }

    #[test]
    fn from_str() {
        assert_eq!(KeywordKind::from_str("let"), Some(KeywordKind::Let));
    }
}
