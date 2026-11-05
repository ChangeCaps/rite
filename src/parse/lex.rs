use crate::{
    diagnostic::{Diagnostic, Reporter, SourceId, Span},
    parse::Token,
};

pub fn lex(reporter: &dyn Reporter, source: SourceId, input: &str) -> Tokens {
    let mut tokens = Vec::new();
    let mut lexer = Lexer::new(reporter, source, input);

    while let Some(token) = lexer.token() {
        tokens.push(token);
    }

    let eof = Span::new(
        source,
        lexer.offset as u32,
        lexer.offset as u32,
    );

    Tokens {
        tokens: tokens.into(),
        eof,
    }
}

pub struct Tokens {
    tokens: Box<[(Token, Span)]>,
    eof:    Span,
}

impl Tokens {
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn nth(&self, n: usize) -> (Token, Span) {
        match self.tokens.get(n) {
            Some(token) => token.clone(),
            None => (Token::Eof, self.eof),
        }
    }
}

struct Lexer<'a> {
    reporter: &'a dyn Reporter,
    source:   SourceId,
    input:    &'a str,
    offset:   usize,
}

impl<'a> Lexer<'a> {
    fn new(reporter: &'a dyn Reporter, source: SourceId, input: &'a str) -> Self {
        Self {
            reporter,
            source,
            input,

            offset: 0,
        }
    }

    fn remaining(&self) -> &'a str {
        &self.input[self.offset..]
    }

    fn peek(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    fn consume(&mut self) -> Span {
        let start = self.offset;

        if let Some(c) = self.peek() {
            self.offset += c.len_utf8();
        }

        let end = self.offset;
        Span::new(self.source, start as u32, end as u32)
    }

    fn consume_while(&mut self, f: impl Fn(char) -> bool) {
        while self.peek().is_some_and(&f) {
            self.consume();
        }
    }

    fn take_while(&mut self, f: impl Fn(char) -> bool) -> (String, Span) {
        let start = self.offset;
        let mut s = String::new();

        while let Some(c) = self.peek()
            && f(c)
        {
            s.push(c);
            self.consume();
        }

        let end = self.offset;
        let span = Span::new(self.source, start as u32, end as u32);

        (s, span)
    }

    fn token(&mut self) -> Option<(Token, Span)> {
        self.whitespace();

        let c = self.peek()?;

        if let Some(token) = self
            .ident(c)
            .or_else(|| self.two_character_symbol())
            .or_else(|| self.one_character_symbol(c))
        {
            return Some(token);
        }

        let span = self.consume();

        let diagnostic =
            Diagnostic::error(format!("unexpected character `{c}`")).label(span, "found here");

        self.reporter.report(diagnostic);

        Some((Token::Other(c.into()), span))
    }

    fn whitespace(&mut self) {
        self.consume_while(|c| c.is_whitespace() && c != '\n');
    }

    fn ident(&mut self, c: char) -> Option<(Token, Span)> {
        if !Self::is_ident_start(c) {
            return None;
        }

        let (ident, span) = self.take_while(Self::is_ident_continue);

        let token = match ident.as_str() {
            "let" => Token::Let,
            _ => Token::Ident(ident),
        };

        Some((token, span))
    }

    fn two_character_symbol(&mut self) -> Option<(Token, Span)> {
        let token = match self.remaining().get(..2).unwrap_or("") {
            "::" => Token::ColonColon,
            "->" => Token::Arrow,
            _ => return None,
        };

        let span = self.consume().join(self.consume());

        Some((token, span))
    }

    fn one_character_symbol(&mut self, c: char) -> Option<(Token, Span)> {
        let token = match c {
            '\n' => Token::Newline,
            '\\' => Token::Backslash,
            '_' => Token::Under,
            '=' => Token::Equal,
            '*' => Token::Star,
            '/' => Token::Slash,
            ':' => Token::Colon,
            ';' => Token::Semi,
            '.' => Token::Dot,
            _ => return None,
        };

        let span = self.consume();

        Some((token, span))
    }

    fn is_ident_start(c: char) -> bool {
        c.is_alphabetic()
    }

    fn is_ident_continue(c: char) -> bool {
        c.is_alphanumeric()
    }
}
