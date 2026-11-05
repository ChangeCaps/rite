use crate::{
    ast::{AppExpr, ErrorExpr, Expr, LamExpr, LetExpr, Parse, VarExpr},
    parse::{Parser, Token},
};

pub fn is_expr(parser: &Parser<'_>) -> bool {
    matches!(
        parser.peek(),
        Token::Ident(..) | Token::Let | Token::Backslash,
    )
}

pub fn expr(parser: &mut Parser<'_>) -> Expr<Parse> {
    match parser.peek() {
        Token::Let => Expr::Let(let_expr(parser)),
        _ => apply_expr(parser),
    }
}

pub fn apply_expr(parser: &mut Parser<'_>) -> Expr<Parse> {
    let mut expr = term_expr(parser);

    while is_expr(parser) {
        let input = term_expr(parser);
        let span = expr.span().join(input.span());

        expr = Expr::App(AppExpr {
            lambda: Box::new(expr),
            input: Box::new(input),
            ty: (),
            span,
        });
    }

    expr
}

pub fn term_expr(parser: &mut Parser<'_>) -> Expr<Parse> {
    match parser.peek() {
        Token::Ident(name) => Expr::Var(item_expr(parser, name)),
        Token::Backslash => Expr::Lam(lambda_expr(parser)),

        _ => {
            let span = parser.expected("expression");
            Expr::Error(ErrorExpr { ty: (), span })
        }
    }
}

pub fn let_expr(parser: &mut Parser<'_>) -> LetExpr<Parse> {
    let span = parser.expect(Token::Let);
    let name = name(parser);

    parser.expect(Token::Equal);

    let input = Box::new(expr(parser));

    parser.expect_any([Token::Semi, Token::Newline]);
    parser.consume_all(Token::Newline);

    let output = Box::new(expr(parser));

    let span = span.join(input.span());

    LetExpr {
        var: name,
        input,
        output,
        ty: (),
        span,
    }
}

pub fn lambda_expr(parser: &mut Parser<'_>) -> LamExpr<Parse> {
    let span = parser.expect(Token::Backslash);
    let input = name(parser);

    parser.expect(Token::Dot);

    let output = Box::new(expr(parser));
    let span = span.join(output.span());

    LamExpr {
        var: input,
        output,
        ty: (),
        span,
    }
}

pub fn item_expr(parser: &mut Parser<'_>, name: String) -> VarExpr<Parse> {
    let span = parser.consume();

    VarExpr {
        var: name,
        ty: (),
        span,
    }
}

pub fn name(parser: &mut Parser<'_>) -> String {
    match parser.peek() {
        Token::Ident(name) => {
            parser.consume();
            name
        }

        _ => {
            parser.expected("identifier");
            String::new()
        }
    }
}
