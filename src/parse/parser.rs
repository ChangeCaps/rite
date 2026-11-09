use core::fmt;

use crate::{
    diagnostic::{Diagnostic, Reporter, Span},
    parse::{Token, Tokens},
    var::Ident,
};

pub struct Parser<'a> {
    reporter: &'a dyn Reporter,
    tokens:   &'a Tokens,
    index:    usize,
}

impl<'a> Parser<'a> {
    pub fn new(reporter: &'a dyn Reporter, tokens: &'a Tokens) -> Self {
        Self {
            reporter,
            tokens,
            index: 0,
        }
    }

    pub fn peek_nth(&self, n: usize) -> Token {
        self.tokens.nth(self.index + n).0
    }

    pub fn peek(&self) -> Token {
        self.peek_nth(0)
    }

    pub fn span(&self) -> Span {
        self.tokens.nth(self.index).1
    }

    pub fn consume(&mut self) -> Span {
        let span = self.span();

        if self.index < self.tokens.len() {
            self.index += 1;
        }

        span
    }

    pub fn consume_while(&mut self, f: impl Fn(Token) -> bool) {
        while f(self.peek()) {
            self.consume();
        }
    }

    pub fn consume_all(&mut self, token: Token) {
        self.consume_while(|actual| actual == token);
    }

    pub fn expect(&mut self, token: Token) -> Span {
        if self.peek() == token {
            self.consume()
        } else {
            self.expected(token)
        }
    }

    pub fn expect_ident(&mut self) -> Option<Ident> {
        match self.peek() {
            Token::Ident(ident) => {
                let span = self.consume();
                Some(Ident::new(ident, span))
            }

            _ => {
                self.expected("identifier");
                None
            }
        }
    }

    pub fn expect_any<const LEN: usize>(&mut self, tokens: [Token; LEN]) -> Span {
        if tokens.iter().any(|token| self.peek() == *token) {
            self.consume()
        } else {
            let token = self.peek();
            let span = self.consume();

            let diagnostic = Diagnostic::error(format!(
                "expected any of {}",
                tokens
                    .iter()
                    .map(Token::to_string)
                    .map(|token| format!("`{token}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
            .label(span, format!("found `{token}`"));

            self.reporter.report(diagnostic);

            span
        }
    }

    pub fn expected(&mut self, expected: impl fmt::Display) -> Span {
        let token = self.peek();
        let span = self.consume();

        let diagnostic = Diagnostic::error(format!("expected `{expected}`"))
            .label(span, format!("found `{token}`"));

        self.reporter.report(diagnostic);

        span
    }
}
