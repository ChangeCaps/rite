use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Ident(String),
    Other(String),

    /* special */
    Eof,

    /* keywords */
    Let,

    /* symbols */
    Equals,
    Backslash,
    Dot,
}

impl Token {
    fn as_str(&self) -> &str {
        match self {
            Token::Ident(ident) => ident,
            Token::Other(other) => other,
            Token::Eof => "end of file",
            Token::Let => "let",
            Token::Equals => "=",
            Token::Backslash => "\\",
            Token::Dot => ".",
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
