use crate::{
    ast::{ConsPat, ErrorPat, Pat},
    parse::{Parser, Token},
};

pub fn is_pat(token: Token) -> bool {
    matches!(token, Token::Ident(..))
}

pub fn pat(parser: &mut Parser<'_>) -> Pat {
    match parser.peek() {
        Token::Ident(..) => cons(parser),
        _ => {
            let span = parser.expected("pattern");
            Pat::Error(ErrorPat { span })
        }
    }
}

pub fn cons(parser: &mut Parser<'_>) -> Pat {
    let span = parser.span();
    let var = parser.expect_ident();

    let mut args = Vec::new();

    while is_pat(parser.peek()) {
        args.push(pat(parser));
    }

    Pat::Cons(ConsPat { var, args, span })
}
