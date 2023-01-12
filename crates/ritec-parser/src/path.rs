use ritec_ast as ast;

use crate::{KeywordKind, Parse, ParseResult, ParseStream, SymbolKind};

impl Parse for ast::ItemSegment {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        // parse the name of the segment
        let name = parser.parse()?;

        // parse the generics
        let mut generics = Vec::new();
        if parser.is(&SymbolKind::Less) {
            parser.expect(&SymbolKind::Less)?;

            loop {
                // if we have reached the end of the generics, break
                if parser.is(&SymbolKind::Greater) {
                    break;
                }

                // parse the type
                generics.push(parser.parse()?);

                // if we have reached the end of the generics, break
                if parser.is(&SymbolKind::Greater) {
                    break;
                }

                parser.expect(&SymbolKind::Comma)?;
            }

            // consume the closing `>`
            parser.expect(&SymbolKind::Greater)?;
        }

        Ok(ast::ItemSegment {
            ident: name,
            generics,
        })
    }
}

impl Parse for ast::PathSegment {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        // parse item segment
        if parser.peek_ident().is_some() {
            Ok(ast::PathSegment::Item(parser.parse()?))

        // parse super segment
        } else if parser.is(&KeywordKind::Super) {
            let span = parser.expect(&KeywordKind::Super)?;
            Ok(ast::PathSegment::SuperSegment(span))

        // parse self segment
        } else if parser.is(&KeywordKind::SelfLower) {
            let span = parser.expect(&KeywordKind::SelfLower)?;
            Ok(ast::PathSegment::SelfSegment(span))
        } else {
            Err(parser.expected("path segment"))
        }
    }
}

impl Parse for ast::Path {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let span = parser.span();

        // parse the absolute flag
        let absolute = if parser.is(&SymbolKind::Colon) {
            parser.expect(&SymbolKind::Colon)?;
            parser.expect(&SymbolKind::Colon)?;

            true
        } else {
            false
        };

        let mut segments = Vec::new();

        // parse segments
        loop {
            // parse the segment
            segments.push(parser.parse()?);

            // if we have reached a `::`, consume it and continue
            if parser.is(&SymbolKind::Colon) {
                parser.expect(&SymbolKind::Colon)?;
                parser.expect(&SymbolKind::Colon)?;

            // otherwise break
            } else {
                break;
            }
        }

        Ok(ast::Path {
            is_absolute: absolute,
            segments,
            span: span | parser.span(),
        })
    }
}
