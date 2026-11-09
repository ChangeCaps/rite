use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Ident(String),
    Other(String),

    /* special */
    Indent,
    Dedent,
    Newline,
    Eof,

    /* keywords */
    Impl,
    Let,
    Match,
    Trait,
    Type,
    Where,

    /* two-character symbols */
    ColonColon,
    EqualEqual,
    BangEqual,
    GtEqual,
    LtEqual,
    PipeGt,
    LtPipe,
    RArrow,
    LArrow,

    /* one-character symbols */
    Under,
    Equal,
    Backslash,
    Pipe,
    Plus,
    Minus,
    Star,
    Slash,
    Colon,
    Semi,
    Dot,
    Gt,
    Lt,

    LParen,
    RParen,
}

impl Token {
    fn as_str(&self) -> &str {
        match self {
            Self::Ident(ident) => ident,
            Self::Other(other) => other,

            /* special */
            Self::Indent => "indent",
            Self::Dedent => "dedent",
            Self::Newline => "newline",
            Self::Eof => "end of file",

            /* keywords */
            Self::Impl => "impl",
            Self::Let => "let",
            Self::Match => "match",
            Self::Trait => "trait",
            Self::Type => "type",
            Self::Where => "where",

            /* two-character symbols */
            Self::ColonColon => "::",
            Self::EqualEqual => "==",
            Self::BangEqual => "!=",
            Self::GtEqual => ">=",
            Self::LtEqual => "<=",
            Self::PipeGt => "|>",
            Self::LtPipe => "<|",
            Self::RArrow => "->",
            Self::LArrow => "<-",

            /* one-character symbols */
            Self::Under => "_",
            Self::Equal => "=",
            Self::Backslash => "\\",
            Self::Pipe => "|",
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Star => "*",
            Self::Slash => "/",
            Self::Colon => ":",
            Self::Semi => ";",
            Self::Dot => ".",
            Self::Gt => ">",
            Self::Lt => "<",

            Self::LParen => "(",
            Self::RParen => ")",
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
