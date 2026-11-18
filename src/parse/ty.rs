use crate::{
    ast::{AppTy, BinTy, ErrorTy, LamTy, ListTy, ParenTy, Ty, VarTy, WhereTy},
    parse::{Parser, Token, binary, kind, operator},
    var::Ident,
};

pub fn is_ty(token: Token) -> bool {
    matches!(
        token,
        Token::Ident(..) | Token::Star | Token::LParen | Token::LBracket | Token::LRParen,
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

        let mut kinds = Vec::new();
        kinds.push(kind(parser));

        while parser.peek() == Token::Plus {
            parser.expect(Token::Plus);
            kinds.push(kind(parser));
        }

        let span = ty.span().join(kinds.last().unwrap().span());

        ty = Ty::Where(WhereTy {
            input: Box::new(ty),
            target: Box::new(input),
            kinds,
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
    binary(parser, apply_ty, |op, lhs, rhs| {
        Ty::Bin(BinTy {
            span: lhs.span().join(rhs.span()),
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            op,
        })
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
        Token::Ident(..) => var_ty(parser),

        Token::LParen => Ty::Paren(paren_ty(parser)),
        Token::LBracket => Ty::List(list_ty(parser)),
        Token::LRParen => Ty::Var(unit_ty(parser)),

        _ => {
            let span = parser.expected("type");
            Ty::Error(ErrorTy { span })
        }
    }
}

pub fn paren_ty(parser: &mut Parser<'_>) -> ParenTy {
    let start = parser.expect(Token::LParen);

    let ty = match parser.peek() {
        Token::LBracket if parser.peek_nth(1) == Token::RBracket => {
            let start = parser.expect(Token::LBracket);
            let end = parser.expect(Token::RBracket);
            let span = start.join(end);

            Ty::Var(VarTy {
                path: Vec::new(),
                var: Ident::new("[]", span),
                span,
            })
        }

        _ if let Some((operator, span)) = operator(parser) => Ty::Var(VarTy {
            path: Vec::new(),
            span,
            var: Ident::operator(operator, span),
        }),

        _ => ty(parser),
    };

    let end = parser.expect(Token::RParen);

    ParenTy {
        ty:   Box::new(ty),
        span: start.join(end),
    }
}

pub fn var_ty(parser: &mut Parser<'_>) -> Ty {
    let span = parser.span();

    let Some(mut var) = parser.expect_ident() else {
        return Ty::Error(ErrorTy { span });
    };

    let mut path = Vec::new();

    while parser.peek() == Token::Dot {
        parser.expect(Token::Dot);

        match parser.expect_ident() {
            Some(ident) => {
                path.push(var);
                var = ident;
            }

            None => {
                return Ty::Error(ErrorTy { span });
            }
        }
    }

    Ty::Var(VarTy { path, var, span })
}

pub fn unit_ty(parser: &mut Parser<'_>) -> VarTy {
    let span = parser.expect(Token::LRParen);

    VarTy {
        path: Vec::new(),
        var: Ident::new("()", span),
        span,
    }
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
