use crate::{
    ast::{
        AppExpr, Assign, BinExpr, ErrorExpr, Expr, ExprField, ExtExpr, LamExpr, LetExpr, ListExpr,
        Lit, LitExpr, MatchArm, MatchExpr, ParenExpr, RecExpr, SeqExpr, VarExpr,
    },
    parse::{Parser, Token, binary, is_pat, operator, operator_from_token, pat},
    var::Ident,
};

pub fn is_expr(parser: &Parser<'_>) -> bool {
    matches!(
        parser.peek(),
        Token::Str(..)
            | Token::Int(..)
            | Token::Real(..)
            | Token::Ident(..)
            | Token::Match
            | Token::Extern
            | Token::Let
            | Token::Back
            | Token::LParen
            | Token::LBracket
            | Token::LRParen,
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
    lines(parser);
    parser.expect(Token::Indent);

    let expr = block(parser);

    lines(parser);
    parser.expect(Token::Dedent);

    expr
}

pub fn block(parser: &mut Parser<'_>) -> Expr {
    match parser.peek() {
        Token::Let => block_let(parser),

        _ => {
            let input = expr(parser);

            if parser.peek() != Token::Newline {
                return input;
            }

            lines(parser);

            if parser.peek() == Token::Dedent {
                return input;
            }

            let output = block(parser);
            let span = input.span().join(output.span());

            Expr::Seq(SeqExpr {
                input: Box::new(input),
                output: Box::new(output),
                span,
            })
        }
    }
}

fn lines(parser: &mut Parser<'_>) {
    parser.consume_while(|token| {
        matches!(
            token,
            Token::Comment(..) | Token::Newline
        )
    })
}

pub fn block_let(parser: &mut Parser<'_>) -> Expr {
    let span = parser.expect(Token::Let);
    let pat = pat(parser, true);

    let assign = match parser.peek() {
        Token::Equal => {
            parser.consume();
            Assign::Assign
        }

        Token::LArrow => {
            parser.consume();
            Assign::Bind
        }

        _ => {
            parser.expect_any([Token::Equal, Token::LArrow]);
            Assign::Assign
        }
    };

    let input = expr(parser);

    lines(parser);

    let output = block(parser);

    Expr::Let(LetExpr {
        pat,
        assign,
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

    while !matches!(
        parser.peek(),
        Token::Dedent | Token::Eof,
    ) {
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
    let pat = pat(parser, true);

    parser.expect(Token::RArrow);

    let output = expr(parser);
    let span = pat.span().join(output.span());

    MatchArm { pat, output, span }
}

pub fn bin_expr(parser: &mut Parser<'_>) -> Expr {
    let mut is_block = false;

    let expr = binary(
        parser,
        |parser| {
            if !is_block && parser.peek() == Token::Newline {
                return block_expr(parser);
            }

            let expr = rec_expr(parser);

            if is_bin_block(parser) && !is_block {
                parser.consume_all(Token::Newline);
                parser.expect(Token::Indent);
                lines(parser);

                is_block = true;
            } else if is_block {
                lines(parser);
            }

            expr
        },
        |op, lhs, rhs| {
            Expr::Bin(BinExpr {
                span: lhs.span().join(rhs.span()),
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            })
        },
    );

    if is_block {
        parser.expect(Token::Dedent);
    }

    expr
}

fn is_bin_block(parser: &Parser<'_>) -> bool {
    let mut n = 0;

    while parser.peek_nth(n) == Token::Newline {
        n += 1;
    }

    parser.peek_nth(n) == Token::Indent && operator_from_token(&parser.peek_nth(n + 1)).is_some()
}

pub fn rec_expr(parser: &mut Parser<'_>) -> Expr {
    let input = app_expr(parser);

    if parser.peek() != Token::LBrace {
        return input;
    }

    parser.expect(Token::LBrace);

    let mut fields = Vec::new();

    if parser.peek() == Token::Newline {
        lines(parser);
        parser.expect(Token::Indent);
        lines(parser);

        while !matches!(
            parser.peek(),
            Token::Dedent | Token::Eof,
        ) {
            fields.push(expr_field(parser));
            lines(parser);
        }

        parser.expect(Token::Dedent);
        lines(parser);
    } else {
        while matches!(parser.peek(), Token::Ident(..)) {
            fields.push(expr_field(parser));

            if parser.peek() != Token::Semi {
                break;
            }

            parser.expect(Token::Semi);
        }
    }

    let end = parser.expect(Token::RBrace);
    let span = input.span().join(end);

    Expr::Rec(RecExpr {
        input: Box::new(input),
        fields,
        span,
    })
}

fn expr_field(parser: &mut Parser<'_>) -> ExprField {
    let start = parser.span();
    let name = parser.expect_ident();

    parser.expect(Token::Equal);

    let expr = expr(parser);
    let span = start.join(expr.span());

    ExprField { name, expr, span }
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
        Token::Ident(..) => item_expr(parser),

        Token::Str(str) => Expr::Lit(str_expr(parser, str)),
        Token::Int(int) => Expr::Lit(int_expr(parser, int)),
        Token::Real(real) => Expr::Lit(real_expr(parser, real)),
        Token::Extern => Expr::Ext(extern_expr(parser)),
        Token::Back => Expr::Lam(lambda_expr(parser)),
        Token::LParen => Expr::Paren(paren_expr(parser)),
        Token::LBracket => Expr::List(list_expr(parser)),
        Token::LRParen => Expr::Var(unit_expr(parser)),

        _ => {
            let span = parser.expected("expression");
            Expr::Error(ErrorExpr { span })
        }
    }
}

pub fn paren_expr(parser: &mut Parser<'_>) -> ParenExpr {
    let start = parser.expect(Token::LParen);

    let expr = match operator(parser) {
        Some((operator, span)) => Expr::Var(VarExpr {
            path: Vec::new(),
            span,
            var: Ident::operator(operator, span),
        }),

        None => expr(parser),
    };

    let end = parser.expect(Token::RParen);

    ParenExpr {
        expr: Box::new(expr),
        span: start.join(end),
    }
}

pub fn str_expr(parser: &mut Parser<'_>, str: String) -> LitExpr {
    let span = parser.consume();

    LitExpr {
        lit: Lit::Str(str),
        span,
    }
}

pub fn int_expr(parser: &mut Parser<'_>, int: i64) -> LitExpr {
    let span = parser.consume();

    LitExpr {
        lit: Lit::Int(int),
        span,
    }
}

pub fn real_expr(parser: &mut Parser<'_>, real: f64) -> LitExpr {
    let span = parser.consume();

    LitExpr {
        lit: Lit::Real(real),
        span,
    }
}

pub fn unit_expr(parser: &mut Parser<'_>) -> VarExpr {
    let span = parser.expect(Token::LRParen);

    VarExpr {
        path: Vec::new(),
        var: Ident::new("()", span),
        span,
    }
}

pub fn extern_expr(parser: &mut Parser<'_>) -> ExtExpr {
    let span = parser.expect(Token::Extern);
    let name = parser.expect_str();

    ExtExpr { name, span }
}

pub fn list_expr(parser: &mut Parser<'_>) -> ListExpr {
    let start = parser.expect(Token::LBracket);

    let mut items = Vec::new();

    if parser.peek() == Token::Newline {
        lines(parser);
        parser.expect(Token::Indent);
        lines(parser);

        while !matches!(
            parser.peek(),
            Token::Dedent | Token::Eof,
        ) {
            items.push(expr(parser));
            lines(parser);
        }

        parser.expect(Token::Dedent);
        lines(parser);
    } else {
        while is_expr(parser) {
            items.push(expr(parser));

            if parser.peek() != Token::Semi {
                break;
            }

            parser.expect(Token::Semi);
        }
    }

    let end = parser.expect(Token::RBracket);

    ListExpr {
        items,
        span: start.join(end),
    }
}

pub fn lambda_expr(parser: &mut Parser<'_>) -> LamExpr {
    let span = parser.expect(Token::Back);

    let mut args = Vec::new();

    while is_pat(parser.peek()) || args.is_empty() {
        args.push(pat(parser, false));
    }

    parser.expect(Token::Dot);

    let output = Box::new(expr(parser));
    let span = span.join(output.span());

    LamExpr { args, output, span }
}

pub fn item_expr(parser: &mut Parser<'_>) -> Expr {
    let span = parser.span();

    let Some(mut var) = parser.expect_ident() else {
        return Expr::Error(ErrorExpr { span });
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
                return Expr::Error(ErrorExpr { span });
            }
        }
    }

    Expr::Var(VarExpr { path, var, span })
}
