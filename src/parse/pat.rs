use crate::{
    ast::{BinPat, ConsPat, ErrorPat, ListPat, ParenPat, Pat, WildPat},
    parse::{Parser, Token, operator},
    var::{Assoc, Operator},
};

pub fn is_pat(token: Token) -> bool {
    matches!(
        token,
        Token::Ident(..) | Token::Under | Token::LParen | Token::LBracket,
    )
}

pub fn pat(parser: &mut Parser<'_>, allow_cons: bool) -> Pat {
    bin_pat(parser, allow_cons)
}

pub fn bin_pat(parser: &mut Parser<'_>, allow_cons: bool) -> Pat {
    let lhs = term_pat(parser, allow_cons);

    let Some((op, _)) = operator(parser) else {
        return lhs;
    };

    let rhs = bin_pat(parser, allow_cons);

    bin(op, lhs, rhs)
}

fn bin(op: Operator, lhs: Pat, rhs: Pat) -> Pat {
    let span = lhs.span().join(rhs.span());

    if let Pat::Bin(ref right) = rhs
        && op.precedence() >= right.op.precedence()
        && op.assoc() == Assoc::Left
        && let Pat::Bin(rhs) = rhs
    {
        return Pat::Bin(BinPat {
            op:   rhs.op,
            lhs:  Box::new(Pat::Bin(BinPat {
                op,
                lhs: Box::new(lhs),
                rhs: rhs.lhs,
                span,
            })),
            rhs:  rhs.rhs,
            span: rhs.span,
        });
    }

    Pat::Bin(BinPat {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
        span,
    })
}

pub fn term_pat(parser: &mut Parser<'_>, allow_cons: bool) -> Pat {
    match parser.peek() {
        Token::Ident(..) => cons_pat(parser, allow_cons),
        Token::Under => wild_pat(parser),
        Token::LParen => paren_pat(parser),
        Token::LBracket => list_pat(parser),

        _ => {
            let span = parser.expected("pattern");
            Pat::Error(ErrorPat { span })
        }
    }
}

pub fn paren_pat(parser: &mut Parser<'_>) -> Pat {
    let start = parser.expect(Token::LParen);
    let pat = pat(parser, true);
    let end = parser.expect(Token::RParen);

    Pat::Paren(ParenPat {
        pat:  Box::new(pat),
        span: start.join(end),
    })
}

pub fn cons_pat(parser: &mut Parser<'_>, allow_cons: bool) -> Pat {
    let span = parser.span();
    let var = parser.expect_ident();

    let mut args = Vec::new();

    while is_pat(parser.peek()) && allow_cons {
        args.push(pat(parser, false));
    }

    Pat::Cons(ConsPat { var, args, span })
}

pub fn wild_pat(parser: &mut Parser<'_>) -> Pat {
    let span = parser.expect(Token::Under);

    Pat::Wild(WildPat { span })
}

pub fn list_pat(parser: &mut Parser<'_>) -> Pat {
    let start = parser.expect(Token::LBracket);
    let end = parser.expect(Token::RBracket);

    Pat::List(ListPat {
        span: start.join(end),
    })
}
