use std::{
    fmt::{self, Display},
    ops::Deref,
};

use ritec_core::Span;

use crate::{ParseStream, Peek};

macro_rules! symbol {
    { $($c:literal $($b:literal)? => $ident:ident),* $(,)? } => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum SymbolKind {
            $($ident),*
        }

        impl SymbolKind {
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$ident => concat!($c $(, $b)?)),*
                }
            }

            pub const fn from_chars(c: char, b: Option<char>) -> Option<Self> {
                match (c, b) {
                    $($(($c, Some($b)) => Some(Self::$ident),)?)*
                    // this a very much a hack but it works
                    $(($c, _) if concat!($($b)?).len() == 0 => Some(Self::$ident),)*
                    _ => None,
                }
            }
        }
    };
}

symbol! {
    /* multi-character symbols */
    '-' '>' => Arrow,
    '=' '>' => FatArrow,
    '=' '=' => EqualEqual,
    '!' '=' => BangEqual,
    '<' '=' => LessEqual,
    '>' '=' => GreaterEqual,
    '+' '=' => PlusEqual,
    '-' '=' => MinusEqual,
    '*' '=' => StarEqual,
    '/' '=' => SlashEqual,
    '%' '=' => PercentEqual,
    '&' '=' => AmpEqual,
    '|' '=' => PipeEqual,
    '^' '=' => CaretEqual,
    '<' '<' => LessLess,
    '>' '>' => GreaterGreater,
    '|' '|' => PipePipe,
    '&' '&' => AmpAmp,

    /* single-character symbols */
    '=' => Equal,
    '+' => Plus,
    '-' => Minus,
    '*' => Star,
    '/' => FSlash,
    '%' => Percent,
    '^' => Caret,
    '&' => Amp,
    '|' => Pipe,
    '!' => Bang,
    '.' => Dot,
    ',' => Comma,
    ':' => Colon,
    ';' => Semicolon,
    '<' => Less,
    '>' => Greater,
}

impl SymbolKind {
    pub const fn is_multi_char(self) -> bool {
        self.as_str().len() > 1
    }

    pub const fn is_single_char(self) -> bool {
        self.as_str().len() == 1
    }
}

impl Display for SymbolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Peek for SymbolKind {
    fn peek(&self, parser: ParseStream) -> bool {
        if let Some(symbol) = parser.peek_symbol() {
            symbol.kind == *self
        } else {
            false
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Symbol {
    kind: SymbolKind,
    span: Span,
}

impl Symbol {
    pub const fn new(kind: SymbolKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub const fn kind(&self) -> SymbolKind {
        self.kind
    }

    pub const fn span(&self) -> Span {
        self.span
    }
}

impl Deref for Symbol {
    type Target = SymbolKind;

    fn deref(&self) -> &Self::Target {
        &self.kind
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str() {
        assert_eq!(SymbolKind::Plus.as_str(), "+");
        assert_eq!(SymbolKind::FatArrow.as_str(), "=>");
    }

    #[test]
    fn from_chars() {
        assert_eq!(SymbolKind::from_chars('+', None), Some(SymbolKind::Plus));
        assert_eq!(
            SymbolKind::from_chars('=', Some('>')),
            Some(SymbolKind::FatArrow)
        );
        assert_eq!(SymbolKind::from_chars('=', None), Some(SymbolKind::Equal));
    }
}
