use crate::{
    ast::{AscribeStmt, AssignStmt, ErrorStmt, Parse, Stmt},
    parse::{Parser, Token, expr, ty},
};

pub fn stmts(parser: &mut Parser<'_>) -> Vec<Stmt<Parse>> {
    let mut stmts = Vec::new();

    parser.consume_all(Token::Newline);

    while parser.peek() != Token::Eof {
        stmts.push(stmt(parser));
        parser.consume_all(Token::Newline);
    }

    stmts
}

pub fn stmt(parser: &mut Parser<'_>) -> Stmt<Parse> {
    match parser.peek() {
        Token::Ident(name) => assign_or_ascribe_stmt(parser, name),

        _ => {
            let span = parser.expected("statement");
            Stmt::Error(ErrorStmt { span })
        }
    }
}

fn assign_or_ascribe_stmt(parser: &mut Parser<'_>, var: String) -> Stmt<Parse> {
    let span = parser.consume();

    match parser.peek() {
        Token::Equal => {
            parser.expect(Token::Equal);

            let expr = expr(parser);
            let span = span.join(expr.span());

            Stmt::Assign(AssignStmt { var, expr, span })
        }

        Token::ColonColon => {
            parser.expect(Token::ColonColon);

            let ty = ty(parser);
            let span = span.join(ty.span());

            Stmt::Ascribe(AscribeStmt { var, ty, span })
        }

        _ => {
            let span = parser.expect_any([Token::Equal, Token::ColonColon]);

            Stmt::Error(ErrorStmt { span })
        }
    }
}
