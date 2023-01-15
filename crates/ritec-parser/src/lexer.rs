use ritec_core::{FileId, FloatLiteral, Ident, IntLiteral, IntPrefix, Literal, Span};

use crate::{Delimiter, Group, Keyword, KeywordKind, Symbol, SymbolKind, TokenStream, TokenTree};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LexerError {
    UnmatchedDelimiter(Delimiter, Span),
    UnexpectedCharacter(char, Span),
}

pub struct Lexer<'a> {
    pub source: &'a str,
    pub index: usize,
    pub file: Option<FileId>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str, file: Option<FileId>) -> Self {
        Self {
            source,
            index: 0,
            file,
        }
    }

    pub fn span(&self) -> Span {
        Span {
            lo: self.index,
            hi: self.index,
            file: self.file,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.index >= self.source.len()
    }

    pub fn peek(&self) -> Option<char> {
        self.source.chars().nth(self.index)
    }

    pub fn peek_nth(&self, n: usize) -> Option<char> {
        self.source.chars().nth(self.index + n)
    }

    pub fn next(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.index += c.len_utf8();
        Some(c)
    }

    pub fn take(&mut self, n: usize) -> String {
        let mut string = String::with_capacity(n);

        for _ in 0..n {
            if let Some(c) = self.next() {
                string.push(c);
            }
        }

        string
    }

    pub fn take_while<F>(&mut self, mut f: F) -> String
    where
        F: FnMut(char) -> bool,
    {
        let mut string = String::new();

        while let Some(c) = self.peek() {
            if f(c) {
                string.push(c);
                self.next();
            } else {
                break;
            }
        }

        string
    }

    pub fn skip_whitespace(&mut self) {
        loop {
            let Some(c) = self.peek() else {
                break;
            };

            if c.is_whitespace() {
                self.next();
            } else {
                break;
            }
        }
    }

    pub fn lex_symbol(&mut self) -> Option<Symbol> {
        let span = self.span();
        let c = self.next()?;

        let kind = SymbolKind::from_chars(c, self.peek())?;

        if kind.is_multi_char() {
            self.next();
        }

        Some(Symbol::new(kind, span | self.span()))
    }

    pub fn lex_identifier(&mut self) -> Ident {
        let span = self.span();
        let mut string = String::new();

        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                string.push(c);
                self.next();
            } else {
                break;
            }
        }

        Ident::new(string, span | self.span())
    }

    pub fn lex_integer(&mut self, radix: u32) -> u64 {
        let mut value = 0u64;

        while let Some(c) = self.peek() {
            if let Some(digit) = c.to_digit(radix) {
                self.next();

                value = value * radix as u64 + digit as u64;
            } else {
                break;
            }
        }

        value
    }

    pub fn lex_number(&mut self) -> Literal {
        let span = self.span();
        let mut prefix = IntPrefix::Dec;

        // lex integer prefix
        if self.peek() == Some('0') {
            if self.peek_nth(1) == Some('x') {
                self.take(2);
                prefix = IntPrefix::Hex;
            } else if self.peek_nth(1) == Some('o') {
                self.take(2);
                prefix = IntPrefix::Oct;
            } else if self.peek_nth(1) == Some('b') {
                self.take(2);
                prefix = IntPrefix::Bin;
            }
        }

        // lex the integral part
        let integer = self.lex_integer(prefix.radix());

        // if the next character is a dot, then we have a floating point number
        if self.peek() == Some('.') {
            self.next();

            let mut fraction = 0.0;
            let mut divisor = 1.0;

            while let Some(c) = self.peek() {
                if let Some(digit) = c.to_digit(prefix.radix()) {
                    self.next();

                    divisor *= prefix.radix() as f64;
                    fraction += digit as f64 / divisor;
                } else {
                    break;
                }
            }

            let lit = FloatLiteral {
                value: integer as f64 + fraction,
                span: span | self.span(),
            };

            Literal::Float(lit)
        } else {
            let lit = IntLiteral {
                prefix,
                value: integer,
                span,
            };

            Literal::Int(lit)
        }
    }

    fn lex(&mut self) -> Result<TokenTree, LexerError> {
        let span = self.span();
        let c = self.peek().unwrap();

        // if we're at a delimiter, lex it as a group
        if let Some(delimitier) = Delimiter::from_open_char(c) {
            let delim_span = span | self.span();
            let mut tokens = Vec::new();

            // we skip the open delimiter
            self.next();

            // then we keep lexing tokens until we hit the closing delimiter.
            // we know delimiters will always be matched because all lexed groups
            // consume the closing delimiter. Therefore we cannot hit it early
            //
            // a - ( + 1 ( b ) / )
            //     ^     ^   ^   ^
            //     |     +---+ <----- inner (nested lex call)
            //     |             |
            //     +-------------+ <- outer (this lex call)
            //
            // we can see that the inner group consumes the closing delimiter
            // and will therefore not be peeked by the loop
            //
            // close char will always be Some if the open char is Some
            loop {
                self.skip_whitespace();

                let Some(c) = self.peek() else {
                    return Err(LexerError::UnmatchedDelimiter(delimitier, delim_span));
                };

                if c == delimitier.close_char().unwrap() {
                    break;
                }

                tokens.push(self.lex()?);
            }

            // we consume the closing delimiter
            self.next();

            // and return the group
            return Ok(TokenTree::Group(Group::new(
                delimitier,
                TokenStream::new(tokens),
                span | self.span(),
            )));
        }

        // if we're an identifier, lex it
        if c.is_alphabetic() || c == '_' {
            let ident = self.lex_identifier();

            if let Some(kind) = KeywordKind::from_str(&ident) {
                let keyword = Keyword::new(kind, ident.span());
                return Ok(TokenTree::Keyword(keyword));
            } else {
                return Ok(TokenTree::Ident(ident));
            }
        }

        // if we're a number, lex it
        if c.is_digit(10) {
            return Ok(TokenTree::Literal(self.lex_number()));
        }

        // if we're a symbol, lex it
        if let Some(symbol) = self.lex_symbol() {
            return Ok(TokenTree::Symbol(symbol));
        }

        Err(LexerError::UnexpectedCharacter(c, span | self.span()))
    }

    pub fn lex_all(&mut self) -> Result<TokenStream, LexerError> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace();

            if self.is_empty() {
                break;
            }

            tokens.push(self.lex()?);
        }

        Ok(TokenStream::new(tokens))
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::TokenStream;

    #[test]
    fn lex() {
        macro_rules! assert_lex {
            { $($source:expr),* $(,)? } => {
                $(assert_eq!(
                    TokenStream::from_str($source)
                        .unwrap()
                        .to_string(),
                    $source
                );)*
            }
        }

        assert_lex! {
            "a",
            "a + b",
            "a + b * c",
            "(a + b)",
            "(a + (b == c))",
        }
    }

    #[test]
    fn fail() {
        macro_rules! assert_fail {
            { $($source:expr),* $(,)? } => {
                $(assert!(TokenStream::from_str($source).is_err());)*
            }
        }

        assert_fail! {
            "[a + b",
            "[(a + b])",
            "{([a + b]}",
            "a + b]",
        }
    }
}
