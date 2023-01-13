use std::fmt::Display;

use ritec_core::{Ident, Span};
use ritec_error::Diagnostic;

use crate::{
    Delimiter, Keyword, Parse, ParseResult, ParseStream, Symbol, SymbolKind, TokenStream, TokenTree,
};

#[derive(Clone)]
pub struct ParseBuffer<'a> {
    stream: &'a TokenStream,
    index: usize,
}

impl<'a> ParseBuffer<'a> {
    pub fn new(stream: &'a TokenStream) -> Self {
        Self { stream, index: 0 }
    }

    pub fn len(&self) -> usize {
        self.stream.len() - self.index
    }

    pub fn is_empty(&self) -> bool {
        self.index >= self.stream.len()
    }

    pub fn parse<T: Parse>(&mut self) -> ParseResult<T> {
        T::parse(self)
    }

    pub fn try_parse<T: Parse>(&mut self) -> Option<T> {
        let index = self.index;

        if let Ok(result) = self.parse() {
            Some(result)
        } else {
            self.index = index;

            None
        }
    }

    pub fn parse_comma_separated<T: Parse>(&mut self) -> ParseResult<Vec<T>> {
        let mut result = Vec::new();

        loop {
            if self.is_empty() {
                break;
            }

            result.push(self.parse()?);

            if self.is_empty() {
                break;
            }

            self.expect(&SymbolKind::Comma)?;
        }

        Ok(result)
    }

    pub fn parse_spanned<T: Parse>(&mut self) -> ParseResult<(T, Span)> {
        let start = self.span();
        let result = self.parse()?;
        Ok((result, start | self.span()))
    }

    pub fn peek(&self) -> Option<&'a TokenTree> {
        self.stream.get(self.index)
    }

    pub fn is<T: Peek>(&mut self, expected: &T) -> bool {
        expected.peek(self)
    }

    pub fn next(&mut self) -> Option<&'a TokenTree> {
        let token = self.stream.get(self.index);
        self.index += 1;
        token
    }

    /// Returns the span at the [`Span::lo`] of the next token, if buffer is at the and, returns
    /// the span at the [`Span::hi`] of the last token.
    ///
    /// If there are no token in `self` [`Span::DUMMY`] will be returned.
    pub fn span(&self) -> Span {
        if self.stream.is_empty() {
            return Span::DUMMY;
        }

        if self.index < self.stream.len() {
            self.stream[self.index].span().shrink_to_lo()
        } else {
            self.stream[self.index - 1].span().shrink_to_hi()
        }
    }

    pub fn peek_ident(&mut self) -> Option<&'a Ident> {
        match self.peek() {
            Some(TokenTree::Ident(ident)) => Some(ident),
            _ => None,
        }
    }

    pub fn peek_symbol(&mut self) -> Option<&'a Symbol> {
        match self.peek() {
            Some(TokenTree::Symbol(symbol)) => Some(symbol),
            _ => None,
        }
    }

    pub fn peek_keyword(&mut self) -> Option<&'a Keyword> {
        match self.peek() {
            Some(TokenTree::Keyword(keyword)) => Some(keyword),
            _ => None,
        }
    }

    pub fn is_blank_ident(&mut self) -> bool {
        self.peek_ident().map_or(false, |ident| ident.is_blank())
    }

    pub fn is_ident(&mut self, expected: &str) -> bool {
        (self.peek_ident()).map_or(false, |ident| ident.value() == expected)
    }

    pub fn expect<T>(&mut self, expected: &T) -> ParseResult<Span>
    where
        T: Peek + Display,
    {
        if !self.is(expected) {
            return Err(self.expected(expected));
        }

        Ok(self.next().unwrap().span())
    }

    pub fn ident(&mut self) -> ParseResult<Ident> {
        let token = self.next().ok_or_else(|| {
            Diagnostic::error("expected identifier")
                .with_message_span("found end of file", self.span())
        })?;

        match token {
            TokenTree::Ident(ident) => Ok(ident.clone()),
            _ => Err(Diagnostic::error("expected identifier")
                .with_message_span(format!("found `{}`", token), token.span())),
        }
    }

    pub fn delim(&mut self, delimiter: Delimiter) -> ParseResult<ParseBuffer<'a>> {
        let c = delimiter.open_char().unwrap();

        let token = self.next().ok_or_else(|| {
            Diagnostic::error(format!("expected `{}`", c))
                .with_message_span("found end of file", self.span())
        })?;

        match token {
            TokenTree::Group(group) if group.delimiter() == delimiter => {
                Ok(ParseBuffer::new(group.stream()))
            }
            _ => Err(Diagnostic::error(format!("expected `{}`", c))
                .with_message_span(format!("found `{}`", token), token.span())),
        }
    }

    pub fn expected<T: Display>(&self, expected: T) -> Diagnostic {
        match self.peek() {
            Some(token) => Diagnostic::error(format!("expected `{}`", expected))
                .with_message_span(format!("found `{}`", token), token.span()),
            None => Diagnostic::error(format!("expected `{}`", expected))
                .with_message_span("found end of file", self.span()),
        }
    }
}

pub trait Peek {
    fn peek(&self, parser: ParseStream) -> bool;
}
