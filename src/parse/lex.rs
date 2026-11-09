use crate::{
    diagnostic::{Diagnostic, Reporter, SourceId, Span},
    parse::Token,
};

pub fn lex(reporter: &dyn Reporter, source: SourceId, input: &str) -> Tokens {
    let lexer = Lexer::new(reporter, source, input);
    lexer.run()
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
    indent:   Vec<String>,
    tokens:   Vec<(Token, Span)>,
}

impl<'a> Lexer<'a> {
    fn new(reporter: &'a dyn Reporter, source: SourceId, input: &'a str) -> Self {
        Self {
            reporter,
            source,
            input,

            offset: 0,
            indent: Vec::new(),
            tokens: Vec::new(),
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

    fn run(mut self) -> Tokens {
        while !self.remaining().is_empty() {
            self.line();
        }

        let eof = Span::new(
            self.source,
            self.offset as u32,
            self.offset as u32,
        );

        for _ in self.indent.drain(..) {
            self.tokens.push((Token::Dedent, eof));
        }

        Tokens {
            tokens: self.tokens.into(),
            eof,
        }
    }

    fn line(&mut self) {
        let newline = if self.peek() == Some('\n') {
            let span = self.consume();
            Some((Token::Newline, span))
        } else {
            None
        };

        self.indent(newline);
        self.whitespace();

        while self.peek() != Some('\n') {
            match self.token() {
                Some(token) => self.tokens.push(token),
                None => return,
            }

            self.whitespace();
        }
    }

    fn indent(&mut self, newline: Option<(Token, Span)>) {
        let (indents, span) = self.whitespace();

        if self.peek() == Some('\n') {
            return;
        }

        let mut indents = indents.as_str();

        for (i, indent) in self.indent.iter().enumerate() {
            if indents.is_empty() {
                for _ in self.indent.drain(i..) {
                    self.tokens.push((Token::Dedent, span));
                }

                self.tokens.extend(newline);

                return;
            }

            if let Some(rest) = indents.strip_prefix(indent) {
                indents = rest;
            } else {
                let diagnostic = Diagnostic::error("invalid indentation").label(span, "here");
                self.reporter.report(diagnostic);
                return;
            }
        }

        self.tokens.extend(newline);

        if !indents.is_empty() {
            self.tokens.push((Token::Indent, span));
            self.indent.push(indents.into());
        }
    }

    fn token(&mut self) -> Option<(Token, Span)> {
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

    fn whitespace(&mut self) -> (String, Span) {
        self.take_while(|c| c.is_whitespace() && c != '\n')
    }

    fn ident(&mut self, c: char) -> Option<(Token, Span)> {
        if !Self::is_ident_start(c) {
            return None;
        }

        let (ident, span) = self.take_while(Self::is_ident_continue);

        let token = match ident.as_str() {
            "impl" => Token::Impl,
            "let" => Token::Let,
            "match" => Token::Match,
            "trait" => Token::Trait,
            "type" => Token::Type,
            "where" => Token::Where,

            _ => Token::Ident(ident),
        };

        Some((token, span))
    }

    fn two_character_symbol(&mut self) -> Option<(Token, Span)> {
        let token = match self.remaining().get(..2).unwrap_or("") {
            "::" => Token::ColonColon,
            "==" => Token::EqualEqual,
            "!=" => Token::BangEqual,
            ">=" => Token::GtEqual,
            "<=" => Token::LtEqual,
            "|>" => Token::PipeGt,
            "<|" => Token::LtPipe,
            "->" => Token::RArrow,
            "<-" => Token::LArrow,
            _ => return None,
        };

        let span = self.consume().join(self.consume());

        Some((token, span))
    }

    fn one_character_symbol(&mut self, c: char) -> Option<(Token, Span)> {
        let token = match c {
            '\\' => Token::Backslash,
            '|' => Token::Pipe,
            '_' => Token::Under,
            '=' => Token::Equal,
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => Token::Slash,
            ':' => Token::Colon,
            ';' => Token::Semi,
            '.' => Token::Dot,
            '>' => Token::Gt,
            '<' => Token::Lt,

            '(' => Token::LParen,
            ')' => Token::RParen,
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
