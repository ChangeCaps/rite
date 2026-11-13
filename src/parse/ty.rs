use crate::{
    ast::{AppTy, BinTy, ErrorTy, LamTy, ListTy, ParenTy, Ty, VarTy, WhereTy},
    parse::{Parser, Token, kind, operator},
    var::{Assoc, Ident, Operator},
};

pub fn is_ty(token: Token) -> bool {
    matches!(
        token,
        Token::Ident(..) | Token::Star | Token::LParen | Token::LBracket,
    )
}

pub fn ty(parser: &mut Parser<'_>) -> Ty {
    where_ty(parser)
}

pub fn where_ty(parser: &mut Parser<'_>) -> Ty {
    let mut ty = lambda_ty(parser);

    while parser.peek() == Token::Where {
        parser.expect(Token::Where);

        let input = lambda_ty(parser);

        parser.expect(Token::ColonColon);

        let kind = kind(parser);
        let span = ty.span().join(kind.span());

        ty = Ty::Where(WhereTy {
            input: Box::new(ty),
            target: Box::new(input),
            kind,
            span,
        });
    }

    ty
}

pub fn lambda_ty(parser: &mut Parser<'_>) -> Ty {
    let ty = bin_ty(parser);

    if parser.peek() == Token::RArrow {
        parser.expect(Token::RArrow);

        let output = lambda_ty(parser);
        let span = ty.span().join(output.span());

        Ty::Lam(LamTy {
            input: Box::new(ty),
            output: Box::new(output),
            span,
        })
    } else {
        ty
    }
}

pub fn bin_ty(parser: &mut Parser<'_>) -> Ty {
    let lhs = apply_ty(parser);

    let Some((op, _)) = operator(parser) else {
        return lhs;
    };

    let rhs = bin_ty(parser);

    bin(op, lhs, rhs)
}

fn bin(op: Operator, lhs: Ty, rhs: Ty) -> Ty {
    let span = lhs.span().join(rhs.span());

    if let Ty::Bin(ref right) = rhs
        && op.precedence() >= right.op.precedence()
        && op.assoc() == Assoc::Left
        && let Ty::Bin(rhs) = rhs
    {
        return Ty::Bin(BinTy {
            op:   rhs.op,
            lhs:  Box::new(Ty::Bin(BinTy {
                op,
                lhs: Box::new(lhs),
                rhs: rhs.lhs,
                span,
            })),
            rhs:  rhs.rhs,
            span: rhs.span,
        });
    }

    Ty::Bin(BinTy {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
        span,
    })
}

pub fn apply_ty(parser: &mut Parser<'_>) -> Ty {
    let mut ty = term_ty(parser);

    while is_ty(parser.peek()) {
        let input = term_ty(parser);
        let span = ty.span().join(input.span());

        ty = Ty::App(AppTy {
            lambda: Box::new(ty),
            input: Box::new(input),
            span,
        });
    }

    ty
}

pub fn term_ty(parser: &mut Parser<'_>) -> Ty {
    match parser.peek() {
        Token::Ident(..) => Ty::Var(var_ty(parser)),

        Token::LParen => Ty::Paren(paren_ty(parser)),
        Token::LBracket => Ty::List(list_ty(parser)),

        _ => {
            let span = parser.expected("type");
            Ty::Error(ErrorTy { span })
        }
    }
}

pub fn paren_ty(parser: &mut Parser<'_>) -> ParenTy {
    let start = parser.expect(Token::LParen);

    let ty = match parser.peek() {
        Token::LBracket => {
            let start = parser.expect(Token::LBracket);
            let end = parser.expect(Token::RBracket);
            let span = start.join(end);

            Ty::Var(VarTy {
                var: Some(Ident::new("[]", span)),
                span,
            })
        }

        _ if let Some((operator, span)) = operator(parser) => Ty::Var(VarTy {
            span,
            var: Some(Ident::operator(operator, span)),
        }),

        _ => ty(parser),
    };

    let end = parser.expect(Token::RParen);

    ParenTy {
        ty:   Box::new(ty),
        span: start.join(end),
    }
}

pub fn var_ty(parser: &mut Parser<'_>) -> VarTy {
    let span = parser.span();
    let var = parser.expect_ident();

    VarTy { var, span }
}

pub fn list_ty(parser: &mut Parser<'_>) -> ListTy {
    let start = parser.expect(Token::LBracket);
    let item = ty(parser);
    let end = parser.expect(Token::RBracket);

    ListTy {
        item: Box::new(item),
        span: start.join(end),
    }
}
