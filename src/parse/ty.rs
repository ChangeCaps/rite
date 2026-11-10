use crate::{
    ast::{AppTy, ErrorTy, LamTy, Ty, VarTy, WhereTy},
    parse::{Parser, Token, kind},
};

pub fn is_ty(token: Token) -> bool {
    matches!(
        token,
        Token::Ident(..) | Token::Star | Token::LParen,
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
    let ty = apply_ty(parser);

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

        Token::LParen => paren_ty(parser),

        _ => {
            let span = parser.expected("type");
            Ty::Error(ErrorTy { span })
        }
    }
}

pub fn paren_ty(parser: &mut Parser<'_>) -> Ty {
    parser.expect(Token::LParen);

    let ty = ty(parser);

    parser.expect(Token::RParen);

    ty
}

pub fn var_ty(parser: &mut Parser<'_>) -> VarTy {
    let span = parser.span();
    let var = parser.expect_ident();

    VarTy { var, span }
}
