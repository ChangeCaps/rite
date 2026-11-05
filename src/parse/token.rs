use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Ident(String),
    Other(String),

    /* special */
    Newline,
    Eof,

    /* keywords */
    Let,

    /* two-character symbols */
    ColonColon,
    Arrow,

    /* one-character symbols */
    Under,
    Equal,
    Backslash,
    Star,
    Slash,
    Colon,
    Semi,
    Dot,
}

impl Token {
    fn as_str(&self) -> &str {
        match self {
            Self::Ident(ident) => ident,
            Self::Other(other) => other,
            Self::Newline => "newline",
            Self::Eof => "end of file",
            Self::Let => "let",

            /* two-character symbols */
            Self::ColonColon => "::",
            Self::Arrow => "->",

            /* one-character symbols */
            Self::Under => "_",
            Self::Equal => "=",
            Self::Backslash => "\\",
            Self::Star => "*",
            Self::Slash => "/",
            Self::Colon => ":",
            Self::Semi => ";",
            Self::Dot => ".",
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
