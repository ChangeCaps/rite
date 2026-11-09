use crate::{
    ast::{AppExpr, BinExpr, ErrorExpr, Expr, LamExpr, LetExpr, MatchArm, MatchExpr, VarExpr},
    parse::{Parser, Token, operator, pat, precedence},
};

pub fn is_expr(parser: &Parser<'_>) -> bool {
    matches!(
        parser.peek(),
        Token::Ident(..) | Token::Match | Token::Let | Token::Backslash | Token::LParen,
    )
}

pub fn expr(parser: &mut Parser<'_>) -> Expr {
    match parser.peek() {
        Token::Match => Expr::Match(match_expr(parser)),
        Token::Newline => block_expr(parser),

        _ => bin_expr(parser),
    }
}

pub fn block_expr(parser: &mut Parser<'_>) -> Expr {
    parser.expect(Token::Newline);
    parser.consume_all(Token::Newline);
    parser.expect(Token::Indent);

    let expr = block(parser);

    parser.consume_all(Token::Newline);
    parser.expect(Token::Dedent);

    expr
}

pub fn block(parser: &mut Parser<'_>) -> Expr {
    match parser.peek() {
        Token::Let => block_let(parser),
        _ => expr(parser),
    }
}

pub fn block_let(parser: &mut Parser<'_>) -> Expr {
    let span = parser.expect(Token::Let);
    let var = parser.expect_ident();

    parser.expect(Token::Equal);

    let input = expr(parser);

    parser.consume_all(Token::Newline);

    let output = block(parser);

    Expr::Let(LetExpr {
        var,
        input: Box::new(input),
        output: Box::new(output),
        span,
    })
}

pub fn match_expr(parser: &mut Parser<'_>) -> MatchExpr {
    let span = parser.expect(Token::Match);
    let input = expr(parser);

    parser.expect(Token::Newline);
    parser.consume_all(Token::Newline);
    parser.expect(Token::Indent);

    let mut arms = Vec::new();

    while parser.peek() != Token::Dedent {
        arms.push(match_arm(parser));
        parser.consume_all(Token::Newline);
    }

    parser.expect(Token::Dedent);

    MatchExpr {
        input: Box::new(input),
        arms,
        span,
    }
}

pub fn match_arm(parser: &mut Parser<'_>) -> MatchArm {
    let pat = pat(parser);

    parser.expect(Token::RArrow);

    let output = expr(parser);
    let span = pat.span().join(output.span());

    MatchArm { pat, output, span }
}

pub fn bin_expr(parser: &mut Parser<'_>) -> Expr {
    let lhs = app_expr(parser);

    let Some(op) = operator(parser) else {
        return lhs;
    };

    let rhs = bin_expr(parser);

    let span = lhs.span().join(rhs.span());

    if let Expr::Bin(ref right) = rhs
        && precedence(&op) > precedence(&right.op)
        && let Expr::Bin(rhs) = rhs
    {
        return Expr::Bin(BinExpr {
            op:   rhs.op,
            lhs:  Box::new(Expr::Bin(BinExpr {
                op,
                lhs: Box::new(lhs),
                rhs: rhs.lhs,
                span,
            })),
            rhs:  rhs.rhs,
            span: rhs.span,
        });
    }

    Expr::Bin(BinExpr {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
        span,
    })
}

pub fn app_expr(parser: &mut Parser<'_>) -> Expr {
    let mut expr = term_expr(parser);

    while is_expr(parser) {
        let input = term_expr(parser);
        let span = expr.span().join(input.span());

        expr = Expr::App(AppExpr {
            lambda: Box::new(expr),
            input: Box::new(input),
            span,
        });
    }

    expr
}

pub fn term_expr(parser: &mut Parser<'_>) -> Expr {
    match parser.peek() {
        Token::Ident(..) => Expr::Var(item_expr(parser)),
        Token::Backslash => Expr::Lam(lambda_expr(parser)),
        Token::LParen => paren_expr(parser),

        _ => {
            let span = parser.expected("expression");
            Expr::Error(ErrorExpr { span })
        }
    }
}

pub fn paren_expr(parser: &mut Parser<'_>) -> Expr {
    parser.expect(Token::LParen);

    let expr = match operator(parser) {
        Some(operator) => Expr::Var(VarExpr {
            span: operator.span(),
            var:  Some(operator),
        }),
        None => expr(parser),
    };

    parser.expect(Token::RParen);

    expr
}

pub fn lambda_expr(parser: &mut Parser<'_>) -> LamExpr {
    let span = parser.expect(Token::Backslash);
    let var = parser.expect_ident();

    parser.expect(Token::Dot);

    let output = Box::new(expr(parser));
    let span = span.join(output.span());

    LamExpr { var, output, span }
}

pub fn item_expr(parser: &mut Parser<'_>) -> VarExpr {
    let span = parser.span();
    let var = parser.expect_ident();

    VarExpr { var, span }
}
