use core::fmt;
use std::{fmt::Display, ops::Index, str::FromStr, sync::Arc};

use ritec_core::{FileId, Ident, Span};

use crate::{Delimiter, Keyword, Lexer, LexerError, Symbol};

/// An immutable list of tokens, that is cheap to clone.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TokenStream {
    tokens: Arc<[TokenTree]>,
}

impl TokenStream {
    pub fn new(token: impl Into<Arc<[TokenTree]>>) -> Self {
        Self {
            tokens: token.into(),
        }
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&TokenTree> {
        self.tokens.get(index)
    }

    pub fn last(&self) -> Option<&TokenTree> {
        self.tokens.last()
    }

    pub fn lex(source: &str, file: Option<FileId>) -> Result<Self, LexerError> {
        let mut lexer = Lexer::new(source, file);
        lexer.lex_all()
    }

    pub fn iter(&self) -> impl Iterator<Item = &TokenTree> {
        self.tokens.iter()
    }
}

impl Index<usize> for TokenStream {
    type Output = TokenTree;

    fn index(&self, index: usize) -> &Self::Output {
        &self.tokens[index]
    }
}

impl FromStr for TokenStream {
    type Err = LexerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lexer = Lexer::new(s, None);
        lexer.lex_all()
    }
}

impl Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tokens: Vec<_> = self.tokens.iter().map(TokenTree::to_string).collect();
        write!(f, "{}", tokens.join(" "))
    }
}

/// A group of tokens, surrounded by delimiters.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Group {
    delimiter: Delimiter,
    stream: TokenStream,
    span: Span,
}

impl Group {
    pub const fn new(delimiter: Delimiter, stream: TokenStream, span: Span) -> Self {
        Self {
            delimiter,
            stream,
            span,
        }
    }

    pub fn stream(&self) -> &TokenStream {
        &self.stream
    }

    pub const fn delimiter(&self) -> Delimiter {
        self.delimiter
    }

    pub const fn span(&self) -> Span {
        self.span
    }

    pub const fn open_span(&self) -> Span {
        self.span().shrink_to_lo().expand_hi(1)
    }

    pub const fn close_span(&self) -> Span {
        self.span().shrink_to_hi().expand_lo(1)
    }
}

impl Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.delimiter {
            Delimiter::Paren => write!(f, "({})", self.stream),
            Delimiter::Brace => write!(f, "{{{}}}", self.stream),
            Delimiter::Bracket => write!(f, "[{}]", self.stream),
            Delimiter::None => write!(f, "{}", self.stream),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TokenTree {
    Ident(Ident),
    Symbol(Symbol),
    Keyword(Keyword),
    Group(Group),
}

impl TokenTree {
    pub fn span(&self) -> Span {
        match self {
            TokenTree::Ident(ident) => ident.span(),
            TokenTree::Symbol(symbol) => symbol.span(),
            TokenTree::Keyword(keyword) => keyword.span(),
            TokenTree::Group(group) => group.span(),
        }
    }
}

impl Display for TokenTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenTree::Ident(ident) => write!(f, "{}", ident),
            TokenTree::Symbol(symbol) => write!(f, "{}", symbol),
            TokenTree::Keyword(keyword) => write!(f, "{}", keyword),
            TokenTree::Group(group) => write!(f, "{}", group),
        }
    }
}
