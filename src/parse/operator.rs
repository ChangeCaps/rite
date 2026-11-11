use crate::{
    parse::{Parser, Token},
    var::Ident,
};

pub fn operator(parser: &mut Parser) -> Option<Ident> {
    let token = parser.peek();
    let kind = operator_from_token(&token)?;
    let span = parser.consume();

    Some(Ident::new(kind, span))
}

pub fn precedence(op: &str) -> u8 {
    match op {
        "|>" | "\\>" | ">>=" | ">>" => 0,

        "<|" | "<\\" | "<<" => 1,

        "==" | "!=" => 2,

        "||" => 3,

        "&&" => 4,

        "+" | "-" => 5,

        "*" | "/" => 6,

        _ => unreachable!("invalid operator `{op}`"),
    }
}

pub fn operator_from_token(token: &Token) -> Option<&str> {
    const OPERATORS: &[Token] = &[
        Token::PipeGt,
        Token::BackGt,
        Token::GtGtEqual,
        Token::GtGt,
        Token::LtBack,
        Token::LtPipe,
        Token::LtLt,
        Token::EqualEqual,
        Token::BangEqual,
        Token::AmpAmp,
        Token::PipePipe,
        Token::Plus,
        Token::Minus,
        Token::Star,
        Token::Slash,
    ];

    OPERATORS.iter().find(|t| *t == token).map(Token::as_str)
}
