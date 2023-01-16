use ritec_ast as ast;

use crate::{KeywordKind, Parse, ParseResult, ParseStream, SymbolKind};

impl Parse for ast::LetStmt {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        // parse the `let` keyword
        let span = parser.expect(&KeywordKind::Let)?;

        // parse the name of the variable
        let name = parser.parse()?;

        // parse the type annotation
        let ty = if parser.is(&SymbolKind::Colon) {
            parser.expect(&SymbolKind::Colon)?;
            Some(parser.parse()?)
        } else {
            None
        };

        // parse the `=` symbol
        let value = if parser.is(&SymbolKind::Equal) {
            parser.expect(&SymbolKind::Equal)?;

            Some(parser.parse()?)
        } else {
            None
        };

        // parse the `;` symbol
        parser.expect(&SymbolKind::Semicolon)?;
        Ok(ast::LetStmt {
            ident: name,
            ty,
            init: value,
            span: span | parser.span(),
        })
    }
}

impl Parse for ast::Stmt {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        if parser.is(&KeywordKind::Let) {
            Ok(ast::Stmt::Let(parser.parse()?))
        } else {
            let expr = parser.parse::<ast::Expr>()?;

            if expr.stmt_needs_semi() {
                parser.expect(&SymbolKind::Semicolon)?;
            }

            Ok(ast::Stmt::Expr(ast::ExprStmt {
                expr,
                span: parser.span(),
            }))
        }
    }
}
