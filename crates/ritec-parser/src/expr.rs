use ritec_ast as ast;

use crate::{Delimiter, KeywordKind, Parse, ParseResult, ParseStream, SymbolKind};

impl Parse for ast::PathExpr {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let (path, span) = parser.parse_spanned()?;
        Ok(ast::PathExpr { path, span })
    }
}

impl Parse for ast::UnaryOp {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        if parser.is(&SymbolKind::Amp) {
            parser.next();
            Ok(ast::UnaryOp::Ref)
        } else if parser.is(&SymbolKind::Star) {
            parser.next();
            Ok(ast::UnaryOp::Deref)
        } else {
            Err(parser.expected("unary operator"))
        }
    }
}

impl Parse for ast::UnaryExpr {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let span = parser.span();
        Ok(ast::UnaryExpr {
            operator: parser.parse()?,
            operand: Box::new(parser.parse()?),
            span: span | parser.span(),
        })
    }
}

impl Parse for ast::ReturnExpr {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let span = parser.expect(&KeywordKind::Return)?;
        let value = if parser.is(&SymbolKind::Semicolon) {
            None
        } else {
            Some(Box::new(parser.parse()?))
        };
        Ok(ast::ReturnExpr { value, span })
    }
}

fn parse_term(parser: ParseStream) -> ParseResult<ast::Expr> {
    if parser.peek_ident().is_some() || parser.is(&SymbolKind::Colon) {
        Ok(ast::Expr::Path(parser.parse()?))
    } else if let Ok(mut contents) = parser.delim(Delimiter::Paren) {
        contents.parse()
    } else {
        Err(parser.expected("expression"))
    }
}

fn parse_unary(parser: ParseStream) -> ParseResult<ast::Expr> {
    if let Some(operator) = parser.try_parse::<ast::UnaryOp>() {
        Ok(ast::Expr::Unary(ast::UnaryExpr {
            operator,
            operand: Box::new(parse_unary(parser)?),
            span: parser.span(),
        }))
    } else {
        parse_term(parser)
    }
}

fn parse_assign(parser: ParseStream) -> ParseResult<ast::Expr> {
    let span = parser.span();
    let expr = parse_unary(parser)?;

    if parser.is(&SymbolKind::Equal) {
        parser.next();

        Ok(ast::Expr::Assign(ast::AssignExpr {
            lhs: Box::new(expr),
            rhs: Box::new(parser.parse()?),
            span: span | parser.span(),
        }))
    } else {
        Ok(expr)
    }
}

impl Parse for ast::Expr {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        if parser.is(&KeywordKind::Return) {
            Ok(ast::Expr::Return(parser.parse()?))
        } else {
            parse_assign(parser)
        }
    }
}
