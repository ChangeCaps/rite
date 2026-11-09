use crate::{
    ast::{ArityKind, ErrorKind, Kind, VarKind},
    parse::{Parser, Token},
};

pub fn kind(parser: &mut Parser<'_>) -> Kind {
    match parser.peek() {
        Token::Ident(..) => Kind::Var(var_kind(parser)),
        Token::Star => Kind::Arity(arity_kind(parser)),

        _ => {
            let span = parser.expected("kind");
            Kind::Error(ErrorKind { span })
        }
    }
}

pub fn var_kind(parser: &mut Parser<'_>) -> VarKind {
    let span = parser.span();
    let var = parser.expect_ident();

    VarKind { var, span }
}

pub fn arity_kind(parser: &mut Parser<'_>) -> ArityKind {
    let mut span = parser.expect(Token::Star);
    let mut arity = 0;

    while parser.peek() == Token::RArrow {
        parser.expect(Token::RArrow);
        span = parser.expect(Token::Star).join(span);
        arity += 1;
    }

    ArityKind { arity, span }
}
