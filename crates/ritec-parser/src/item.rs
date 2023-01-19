use ritec_ast as ast;

use crate::{Delimiter, KeywordKind, Parse, ParseResult, ParseStream, SymbolKind};

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

impl Parse for ast::Field {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let ident = parser.parse()?;
        parser.expect(&SymbolKind::Colon)?;
        let ty = parser.parse()?;
        parser.expect(&SymbolKind::Comma)?;

        Ok(ast::Field {
            ident,
            ty,
            span: parser.span(),
        })
    }
}

impl Parse for ast::SelfArgument {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        if parser.is(&SymbolKind::Amp) || parser.is(&SymbolKind::Star) {
            parser.next();
            parser.expect(&KeywordKind::SelfLower)?;
            Ok(ast::SelfArgument::Pointer)
        } else {
            parser.expect(&KeywordKind::SelfLower)?;
            Ok(ast::SelfArgument::Owned)
        }
    }
}

impl Parse for ast::Method {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let span = parser.expect(&KeywordKind::Fn)?;
        let ident = parser.parse()?;
        let generics = parser.parse()?;
        let mut contents = parser.delim(Delimiter::Paren)?;
        let self_argument = contents.try_parse();

        if self_argument.is_some() && !contents.is_empty() {
            contents.expect(&SymbolKind::Comma)?;
        }

        let arguments = contents.parse_comma_separated()?;

        let return_type = if parser.is(&SymbolKind::Arrow) {
            parser.next();
            Some(parser.parse()?)
        } else {
            None
        };

        Ok(ast::Method {
            ident,
            generics,
            self_argument,
            arguments,
            return_type,
            body: parser.parse()?,
            span: span | parser.span(),
        })
    }
}

impl Parse for ast::Class {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let span = parser.expect(&KeywordKind::Class)?;
        let ident = parser.parse()?;
        let generics = parser.parse()?;

        let mut contents = parser.delim(Delimiter::Brace)?;
        let mut fields = Vec::new();
        let mut methods = Vec::new();

        while !contents.is_empty() {
            if contents.is(&KeywordKind::Fn) {
                methods.push(contents.parse()?);
            } else {
                fields.push(contents.parse()?);
            }
        }

        Ok(ast::Class {
            module: parser.module(),
            ident,
            generics,
            fields,
            methods,
            span: span | parser.span(),
        })
    }
}

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
            parser.next();
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

impl Parse for ast::Item {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        if parser.is(&KeywordKind::Mod) {
            Ok(ast::Item::Module(parser.parse()?))
        } else if parser.is(&KeywordKind::Class) {
            Ok(ast::Item::Class(parser.parse()?))
        } else if parser.is(&KeywordKind::Fn) {
            Ok(ast::Item::Function(parser.parse()?))
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
