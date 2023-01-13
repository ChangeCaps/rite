use ritec_ast as ast;
use ritec_core::{FloatSize, IntSize};

use crate::{Delimiter, KeywordKind, Parse, ParseResult, ParseStream, SymbolKind};

impl Parse for ast::VoidType {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let span = parser.expect(&KeywordKind::Void)?;
        Ok(ast::VoidType { span })
    }
}

impl Parse for ast::BoolType {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let span = parser.expect(&KeywordKind::Bool)?;
        Ok(ast::BoolType { span })
    }
}

impl Parse for ast::IntType {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let (signed, size) = if parser.is(&KeywordKind::I8) {
            (true, Some(IntSize::I8))
        } else if parser.is(&KeywordKind::I16) {
            (true, Some(IntSize::I16))
        } else if parser.is(&KeywordKind::I32) {
            (true, Some(IntSize::I32))
        } else if parser.is(&KeywordKind::I64) {
            (true, Some(IntSize::I64))
        } else if parser.is(&KeywordKind::I128) {
            (true, Some(IntSize::I128))
        } else if parser.is(&KeywordKind::Isize) {
            (true, None)
        } else if parser.is(&KeywordKind::U8) {
            (false, Some(IntSize::I8))
        } else if parser.is(&KeywordKind::U16) {
            (false, Some(IntSize::I16))
        } else if parser.is(&KeywordKind::U32) {
            (false, Some(IntSize::I32))
        } else if parser.is(&KeywordKind::U64) {
            (false, Some(IntSize::I64))
        } else if parser.is(&KeywordKind::U128) {
            (false, Some(IntSize::I128))
        } else if parser.is(&KeywordKind::Usize) {
            (false, None)
        } else {
            return Err(parser.expected("integer type"));
        };

        let span = parser.next().unwrap().span();

        Ok(ast::IntType { signed, size, span })
    }
}

impl Parse for ast::FloatType {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let size = if parser.is(&KeywordKind::F16) {
            FloatSize::F16
        } else if parser.is(&KeywordKind::F32) {
            FloatSize::F32
        } else if parser.is(&KeywordKind::F64) {
            FloatSize::F64
        } else {
            return Err(parser.expected("float type"));
        };

        let span = parser.next().unwrap().span();

        Ok(ast::FloatType { size, span })
    }
}

impl Parse for ast::PointerType {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        parser.expect(&SymbolKind::Star)?;
        let (pointee, span) = parser.parse_spanned()?;
        Ok(ast::PointerType {
            pointee: Box::new(pointee),
            span,
        })
    }
}

impl Parse for ast::SliceType {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let span = parser.span();
        let mut content = parser.delim(Delimiter::Bracket)?;
        Ok(ast::SliceType {
            element: Box::new(content.parse()?),
            span: span | parser.span(),
        })
    }
}

impl Parse for ast::FunctionType {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let span = parser.span();
        parser.expect(&KeywordKind::Fn)?;
        let mut content = parser.delim(Delimiter::Paren)?;

        let return_type = if parser.is(&SymbolKind::Arrow) {
            parser.next();
            parser.parse::<ast::Type>()?
        } else {
            ast::Type::Void(ast::VoidType {
                span: span | parser.span(),
            })
        };

        Ok(ast::FunctionType {
            arguments: content.parse_comma_separated::<ast::Type>()?,
            return_type: Box::new(return_type),
            span: span | parser.span(),
        })
    }
}

impl Parse for ast::TupleType {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let span = parser.span();
        let mut content = parser.delim(Delimiter::Paren)?;
        let elements = content.parse_comma_separated::<ast::Type>()?;
        Ok(ast::TupleType {
            fields: elements,
            span: span | parser.span(),
        })
    }
}

impl Parse for ast::PathType {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let path = parser.parse::<ast::Path>()?;
        Ok(ast::PathType {
            span: path.span,
            path,
        })
    }
}

impl Parse for ast::Type {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        // parse inferred type
        if parser.is_blank_ident() {
            let span = parser.next().unwrap().span();
            Ok(ast::Type::Inferred(ast::InferredType { span }))

        // parse void type
        } else if let Some(ty) = parser.try_parse() {
            Ok(ast::Type::Void(ty))

        // parse bool type
        } else if let Some(ty) = parser.try_parse() {
            Ok(ast::Type::Bool(ty))

        // parse integer type
        } else if let Some(ty) = parser.try_parse() {
            Ok(ast::Type::Int(ty))

        // parse float type
        } else if let Some(ty) = parser.try_parse() {
            Ok(ast::Type::Float(ty))

        // parse pointer type
        } else if let Some(ty) = parser.try_parse() {
            Ok(ast::Type::Pointer(ty))

        // parse slice type
        } else if let Some(ty) = parser.try_parse() {
            Ok(ast::Type::Slice(ty))

        // parse function type
        } else if let Some(ty) = parser.try_parse() {
            Ok(ast::Type::Function(ty))

        // parse tuple type
        } else if let Some(ty) = parser.try_parse() {
            Ok(ast::Type::Tuple(ty))

        // parse path type
        } else if let Some(ty) = parser.try_parse() {
            Ok(ast::Type::Path(ty))
        } else {
            return Err(parser.expected("type"));
        }
    }
}
