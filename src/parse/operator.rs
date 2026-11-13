use crate::{
    diagnostic::Span,
    parse::{Parser, Token},
    var::Operator,
};

pub fn operator(parser: &mut Parser) -> Option<(Operator, Span)> {
    let token = parser.peek();
    let operator = operator_from_token(&token)?;
    let span = parser.consume();

    Some((operator, span))
}

pub fn operator_from_token(token: &Token) -> Option<Operator> {
    Operator::ALL
        .iter()
        .find(|op| *op.as_token() == *token)
        .cloned()
}
