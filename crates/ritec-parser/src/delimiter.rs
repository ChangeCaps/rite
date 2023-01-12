use std::fmt::{self, Display};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Delimiter {
    Paren,
    Brace,
    Bracket,
    None,
}

impl Delimiter {
    pub const fn from_open_char(c: char) -> Option<Self> {
        match c {
            '(' => Some(Self::Paren),
            '{' => Some(Self::Brace),
            '[' => Some(Self::Bracket),
            _ => None,
        }
    }

    pub const fn from_close_char(c: char) -> Option<Self> {
        match c {
            ')' => Some(Self::Paren),
            '}' => Some(Self::Brace),
            ']' => Some(Self::Bracket),
            _ => None,
        }
    }

    pub const fn char_size(self) -> usize {
        match self {
            Delimiter::None => 0,
            _ => 1,
        }
    }

    pub const fn open_char(self) -> Option<char> {
        match self {
            Delimiter::Paren => Some('('),
            Delimiter::Brace => Some('{'),
            Delimiter::Bracket => Some('['),
            Delimiter::None => None,
        }
    }

    pub const fn close_char(self) -> Option<char> {
        match self {
            Delimiter::Paren => Some(')'),
            Delimiter::Brace => Some('}'),
            Delimiter::Bracket => Some(']'),
            Delimiter::None => None,
        }
    }
}

impl Display for Delimiter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(c) = self.open_char() {
            write!(f, "{}", c)
        } else {
            Ok(())
        }
    }
}
