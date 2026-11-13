use crate::{
    ast::{
        AscribeStmt, DefineStmt, ErrorStmt, ImplMember, ImplStmt, ImportStmt, Pat, RecordCons,
        RecordField, Stmt, TraitMember, TraitStmt, TupleCons, TypeCons, TypeStmt,
    },
    parse::{Parser, Token, expr, is_pat, is_ty, kind, operator, pat, term_ty, ty},
    var::Ident,
};

pub fn stmts(parser: &mut Parser<'_>) -> Vec<Stmt> {
    let mut stmts = Vec::new();

    lines(parser);

    while parser.peek() != Token::Eof {
        stmts.push(stmt(parser));
        lines(parser);
    }

    stmts
}

pub fn lines(parser: &mut Parser<'_>) {
    parser.consume_while(|token| {
        matches!(
            token,
            Token::Comment(..) | Token::Newline,
        )
    });
}

pub fn stmt(parser: &mut Parser<'_>) -> Stmt {
    match parser.peek() {
        Token::Import => import_stmt(parser),
        Token::Impl => impl_stmt(parser),
        Token::Type => type_stmt(parser),
        Token::Trait => trait_stmt(parser),

        _ => define_or_ascribe_stmt(parser),
    }
}

pub fn import_stmt(parser: &mut Parser<'_>) -> Stmt {
    let span = parser.expect(Token::Import);
    let path = parser.expect_str();

    let alias = if parser.peek() == Token::As {
        parser.consume();
        parser.expect_ident()
    } else {
        None
    };

    Stmt::Import(ImportStmt { path, alias, span })
}

pub fn define_or_ascribe_stmt(parser: &mut Parser<'_>) -> Stmt {
    let span = parser.span();
    let var = ident_or_operator(parser);

    let args = args(parser);

    match parser.peek() {
        Token::Equal => {
            parser.expect(Token::Equal);

            let expr = expr(parser);
            let span = span.join(expr.span());

            Stmt::Define(DefineStmt {
                var,
                args,
                expr,
                span,
            })
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

pub fn ident_or_operator(parser: &mut Parser<'_>) -> Option<Ident> {
    match parser.peek() {
        Token::LParen => {
            parser.expect(Token::LParen);
            let (operator, span) = operator(parser)?;
            parser.expect(Token::RParen);

            Some(Ident::operator(operator, span))
        }

        Token::LBracket => {
            let start = parser.expect(Token::LBracket);
            let end = parser.expect(Token::RBracket);
            let span = start.join(end);

            Some(Ident::new("[]", span))
        }

        _ => parser.expect_ident(),
    }
}

pub fn type_stmt(parser: &mut Parser<'_>) -> Stmt {
    let span = parser.expect(Token::Type);
    let var = ident_or_operator(parser);

    let mut input = Vec::new();

    while let Token::Ident(..) = parser.peek() {
        let var = parser.expect_ident();
        input.push(var);
    }

    parser.expect(Token::Equal);

    let cons = type_cons(parser);

    Stmt::Type(TypeStmt {
        var,
        input,
        cons,
        span,
    })
}

pub fn type_cons(parser: &mut Parser<'_>) -> TypeCons {
    if matches!(parser.peek(), Token::Ident(..)) && parser.peek_nth(1) == Token::LBrace {
        TypeCons::Record(rec_cons(parser))
    } else {
        let mut cons = Vec::new();

        cons.push(self::tuple_cons(parser));

        while parser.peek() == Token::Pipe {
            parser.expect(Token::Pipe);
            cons.push(self::tuple_cons(parser));
        }

        TypeCons::Tuple(cons)
    }
}

pub fn rec_cons(parser: &mut Parser<'_>) -> RecordCons {
    let start = parser.span();
    let var = parser.expect_ident();

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
            fields.push(rec_field(parser));
            lines(parser);
        }

        parser.expect(Token::Dedent);
        lines(parser);
    } else {
        while matches!(parser.peek(), Token::Ident(..)) {
            fields.push(rec_field(parser));

            if parser.peek() != Token::Semi {
                break;
            }

            parser.expect(Token::Semi);
        }
    }

    let end = parser.expect(Token::RBrace);
    let span = start.join(end);

    RecordCons { var, fields, span }
}

fn rec_field(parser: &mut Parser<'_>) -> RecordField {
    let start = parser.span();
    let name = parser.expect_ident();

    parser.expect(Token::ColonColon);

    let ty = ty(parser);

    let span = start.join(ty.span());

    RecordField { name, ty, span }
}

pub fn tuple_cons(parser: &mut Parser<'_>) -> TupleCons {
    let span = parser.span();
    let var = ident_or_operator(parser);

    let mut args = Vec::new();

    while is_ty(parser.peek()) {
        args.push(term_ty(parser));
    }

    TupleCons { var, args, span }
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
    let var = ident_or_operator(parser);

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

        while !matches!(
            parser.peek(),
            Token::Dedent | Token::Eof
        ) {
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
    let var = ident_or_operator(parser);

    let args = args(parser);

    parser.expect(Token::Equal);

    let expr = expr(parser);
    let span = span.join(expr.span());

    ImplMember::Assign(DefineStmt {
        var,
        args,
        expr,
        span,
    })
}

fn args(parser: &mut Parser<'_>) -> Vec<Pat> {
    let mut args = Vec::new();

    while is_pat(parser.peek()) {
        args.push(pat(parser, false));
    }

    args
}

fn is_indent(parser: &Parser<'_>) -> bool {
    let mut n = 0;

    while parser.peek_nth(n) == Token::Newline {
        n += 1;
    }

    parser.peek_nth(n) == Token::Indent
}
