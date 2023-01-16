use ritec_ast as ast;

use crate::{Delimiter, KeywordKind, Parse, ParseResult, ParseStream, SymbolKind};

impl Parse for ast::FunctionArgument {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let span = parser.span();

        let ident = parser.parse()?;
        parser.expect(&SymbolKind::Colon)?;
        let ty = parser.parse()?;

        Ok(ast::FunctionArgument {
            ident,
            ty,
            span: span | parser.span(),
        })
    }
}

impl Parse for ast::Function {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let span = parser.expect(&KeywordKind::Fn)?;

        let ident = parser.parse()?;
        let generics = parser.parse()?;

        let mut content = parser.delim(Delimiter::Paren)?;
        let arguments = content.parse_comma_separated()?;

        let return_type = if parser.is(&SymbolKind::Arrow) {
            parser.expect(&SymbolKind::Arrow)?;
            Some(parser.parse()?)
        } else {
            None
        };

        let body = parser.parse()?;

        Ok(ast::Function {
            module: parser.module(),
            ident,
            generics,
            arguments,
            return_type,
            body,
            span: span | parser.span(),
        })
    }
}

impl Parse for ast::ModuleItem {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let span = parser.expect(&KeywordKind::Mod)?;
        let ident = parser.parse()?;
        parser.expect(&SymbolKind::Semicolon)?;

        Ok(ast::ModuleItem {
            ident,
            span: span | parser.span(),
        })
    }
}

impl Parse for ast::Item {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        if parser.is(&KeywordKind::Fn) {
            Ok(ast::Item::Function(parser.parse()?))
        } else if parser.is(&KeywordKind::Mod) {
            Ok(ast::Item::Module(parser.parse()?))
        } else {
            Err(parser.expected("item"))
        }
    }
}

impl Parse for ast::Items {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let span = parser.span();
        let mut items = Vec::new();
        while !parser.is_empty() {
            items.push(parser.parse()?);
        }

        Ok(ast::Items {
            items,
            span: span | parser.span(),
        })
    }
}
