use ritec_ast as ast;

use crate::{Delimiter, Parse, ParseResult, ParseStream, SymbolKind};

impl Parse for ast::PathExpr {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        Ok(ast::PathExpr {
            path: parser.parse()?,
        })
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
            operand: Box::new(parser.parse()?),
            span: parser.span(),
        }))
    } else {
        parse_term(parser)
    }
}

impl Parse for ast::Expr {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        parse_unary(parser)
    }
}
