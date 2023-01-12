use ritec_ast as ast;

use crate::{Parse, ParseResult, ParseStream, SymbolKind};

impl Parse for ast::GenericParameter {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let span = parser.span();
        Ok(ast::GenericParameter {
            ident: parser.parse()?,
            span: span | parser.span(),
        })
    }
}

impl Parse for ast::Generics {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let span = parser.span();

        // if we're at a `<`, parse the generics, otherwise return an empty list
        let mut params = Vec::new();
        if parser.is(&SymbolKind::Less) {
            parser.expect(&SymbolKind::Less)?;

            // until we reach a '>' try to parse a generics parameter, followed by a ','
            loop {
                if parser.is(&SymbolKind::Greater) {
                    break;
                }

                params.push(parser.parse()?);

                if parser.is(&SymbolKind::Greater) {
                    break;
                }

                parser.expect(&SymbolKind::Comma)?;
            }

            parser.expect(&SymbolKind::Greater)?;
        }

        Ok(ast::Generics {
            params,
            span: span | parser.span(),
        })
    }
}
