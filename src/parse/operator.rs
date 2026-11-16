use crate::{
    diagnostic::Span,
    parse::{Parser, Token},
    var::{Assoc, Operator},
};

pub fn binary<T>(
    parser: &mut Parser,
    mut parse: impl FnMut(&mut Parser) -> T,
    build: impl Fn(Operator, T, T) -> T,
) -> T {
    fn should_pop(lhs: Operator, rhs: Operator) -> bool {
        lhs.precedence() > rhs.precedence()
            || lhs.precedence() == rhs.precedence() && lhs.assoc() == Assoc::Left
    }

    fn reduce<T>(
        operands: &mut Vec<T>,
        operators: &mut Vec<Operator>,
        build: impl Fn(Operator, T, T) -> T,
    ) {
        let op = operators.pop().unwrap();

        let rhs = operands.pop().unwrap();
        let lhs = operands.pop().unwrap();

        operands.push(build(op, lhs, rhs));
    }

    let mut operands: Vec<T> = vec![parse(parser)];
    let mut operators: Vec<Operator> = Vec::new();

    while let Some((rhs, _)) = operator(parser) {
        while let Some(lhs) = operators.last()
            && should_pop(*lhs, rhs)
        {
            reduce(&mut operands, &mut operators, &build);
        }

        operators.push(rhs);
        operands.push(parse(parser));
    }

    while !operators.is_empty() {
        reduce(&mut operands, &mut operators, &build);
    }

    operands.pop().unwrap()
}

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
