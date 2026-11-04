use core::fmt;

use crate::{
    diagnostic::{Diagnostic, Reporter, Span},
    parse::{Token, Tokens},
};

pub struct Parser<'a> {
    reporter: &'a dyn Reporter,
    tokens: &'a Tokens,
    index: usize,
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

    pub fn expected(&mut self, expected: impl fmt::Display) -> Span {
        let token = self.peek();
        let span = self.span();

        let diagnostic = Diagnostic::error(format!("expected `{expected}`"))
            .label(span, format!("found `{token}`"));

        self.reporter.report(diagnostic);

        span
    }
}
