use crate::{
    parse::{Parser, Token},
    var::Ident,
};

pub fn operator(parser: &mut Parser) -> Option<Ident> {
    let token = parser.peek();
    let kind = from_token(token)?;
    let span = parser.consume();

    Some(Ident::new(kind, span))
}

pub fn precedence(op: &str) -> u8 {
    match op {
        "|>" => 0,
        "<|" => 1,

        "+" => 2,
        "-" => 2,

        "*" => 3,
        "/" => 3,

        _ => unreachable!("invalid operator `{op}`"),
    }
}

fn from_token(token: Token) -> Option<&'static str> {
    Some(match token {
        Token::PipeGt => "|>",
        Token::LtPipe => "<|",

        Token::Plus => "+",
        Token::Minus => "-",

        Token::Star => "*",
        Token::Slash => "/",

        _ => return None,
    })
}
