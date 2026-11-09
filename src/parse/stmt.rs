use crate::{
    ast::{
        AscribeStmt, DefineStmt, ErrorStmt, ImplMember, ImplStmt, Stmt, TraitMember, TraitStmt,
        TypeCons, TypeStmt,
    },
    parse::{Parser, Token, expr, is_ty, kind, operator, term_ty, ty},
    var::Ident,
};

pub fn stmts(parser: &mut Parser<'_>) -> Vec<Stmt> {
    let mut stmts = Vec::new();

    parser.consume_all(Token::Newline);

    while parser.peek() != Token::Eof {
        stmts.push(stmt(parser));
        parser.consume_all(Token::Newline);
    }

    stmts
}

pub fn stmt(parser: &mut Parser<'_>) -> Stmt {
    match parser.peek() {
        Token::Type => type_stmt(parser),
        Token::Trait => trait_stmt(parser),
        Token::Impl => impl_stmt(parser),

        _ => define_or_ascribe_stmt(parser),
    }
}

pub fn define_or_ascribe_stmt(parser: &mut Parser<'_>) -> Stmt {
    let span = parser.span();
    let var = definee(parser);

    match parser.peek() {
        Token::Equal => {
            parser.expect(Token::Equal);

            let expr = expr(parser);
            let span = span.join(expr.span());

            Stmt::Define(DefineStmt { var, expr, span })
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

pub fn definee(parser: &mut Parser<'_>) -> Option<Ident> {
    match parser.peek() {
        Token::LParen => {
            parser.expect(Token::LParen);
            let operator = operator(parser);
            parser.expect(Token::RParen);
            operator
        }

        _ => parser.expect_ident(),
    }
}

pub fn type_stmt(parser: &mut Parser<'_>) -> Stmt {
    let span = parser.expect(Token::Type);
    let var = parser.expect_ident();

    let mut input = Vec::new();

    while let Token::Ident(..) = parser.peek() {
        let var = parser.expect_ident();
        input.push(var);
    }

    let mut cons = Vec::new();

    if parser.peek() == Token::Equal {
        parser.expect(Token::Equal);

        cons.push(self::type_cons(parser));

        while parser.peek() == Token::Pipe {
            parser.expect(Token::Pipe);
            cons.push(self::type_cons(parser));
        }
    }

    Stmt::Type(TypeStmt {
        var,
        input,
        cons,
        span,
    })
}

pub fn type_cons(parser: &mut Parser<'_>) -> TypeCons {
    let span = parser.span();
    let var = parser.expect_ident();

    let mut args = Vec::new();

    while is_ty(parser.peek()) {
        args.push(term_ty(parser));
    }

    TypeCons { var, args, span }
}

pub fn trait_stmt(parser: &mut Parser<'_>) -> Stmt {
    let span = parser.expect(Token::Trait);

    let var = parser.expect_ident();
    let input = parser.expect_ident();

    let kind = if parser.peek() == Token::ColonColon {
        parser.consume();
        Some(kind(parser))
    } else {
        None
    };

    let mut members = Vec::new();

    if is_indent(parser) {
        parser.consume_all(Token::Newline);
        parser.expect(Token::Indent);

        while parser.peek() != Token::Dedent {
            members.push(trait_member(parser));
            parser.consume_all(Token::Newline);
        }

        parser.expect(Token::Dedent);
    }

    Stmt::Trait(TraitStmt {
        var,
        input,
        kind,
        members,
        span,
    })
}

pub fn trait_member(parser: &mut Parser<'_>) -> TraitMember {
    let span = parser.span();
    let var = definee(parser);

    parser.expect(Token::ColonColon);

    let ty = ty(parser);
    let span = span.join(ty.span());

    TraitMember::Ascribe(AscribeStmt { var, ty, span })
}

pub fn impl_stmt(parser: &mut Parser<'_>) -> Stmt {
    let span = parser.expect(Token::Impl);
    let var = parser.expect_ident();
    let input = ty(parser);

    let mut members = Vec::new();

    if is_indent(parser) {
        parser.consume_all(Token::Newline);
        parser.expect(Token::Indent);

        while parser.peek() != Token::Dedent {
            members.push(impl_member(parser));
            parser.consume_all(Token::Newline);
        }

        parser.expect(Token::Dedent);
    }

    Stmt::Impl(ImplStmt {
        var,
        input,
        members,
        span,
    })
}

pub fn impl_member(parser: &mut Parser<'_>) -> ImplMember {
    let span = parser.span();
    let var = definee(parser);

    parser.expect(Token::Equal);

    let expr = expr(parser);
    let span = span.join(expr.span());

    ImplMember::Assign(DefineStmt { var, expr, span })
}

fn is_indent(parser: &Parser<'_>) -> bool {
    let mut n = 0;

    while parser.peek_nth(n) == Token::Newline {
        n += 1;
    }

    parser.peek_nth(n) == Token::Indent
}
