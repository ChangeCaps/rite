use ritec_ast as ast;

use crate::{Delimiter, KeywordKind, Parse, ParseResult, ParseStream, SymbolKind};

impl Parse for ast::IntType {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let result = if parser.is(&KeywordKind::I8) {
            Self::I8
        } else if parser.is(&KeywordKind::I16) {
            Self::I16
        } else if parser.is(&KeywordKind::I32) {
            Self::I32
        } else if parser.is(&KeywordKind::I64) {
            Self::I64
        } else if parser.is(&KeywordKind::I128) {
            Self::I128
        } else if parser.is(&KeywordKind::Isize) {
            Self::Isize
        } else if parser.is(&KeywordKind::U8) {
            Self::U8
        } else if parser.is(&KeywordKind::U16) {
            Self::U16
        } else if parser.is(&KeywordKind::U32) {
            Self::U32
        } else if parser.is(&KeywordKind::U64) {
            Self::U64
        } else if parser.is(&KeywordKind::U128) {
            Self::U128
        } else if parser.is(&KeywordKind::Usize) {
            Self::Usize
        } else {
            return Err(parser.expected("integer type"));
        };

        parser.next();

        Ok(result)
    }
}

impl Parse for ast::FloatType {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let result = if parser.is(&KeywordKind::F16) {
            Self::F16
        } else if parser.is(&KeywordKind::F32) {
            Self::F32
        } else if parser.is(&KeywordKind::F64) {
            Self::F64
        } else {
            return Err(parser.expected("float type"));
        };

        parser.next();

        Ok(result)
    }
}

impl Parse for ast::PointerType {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        parser.expect(&SymbolKind::Star)?;
        Ok(ast::PointerType::new(parser.parse::<ast::Type>()?))
    }
}

impl Parse for ast::SliceType {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let mut content = parser.delim(Delimiter::Bracket)?;
        Ok(ast::SliceType::new(content.parse::<ast::Type>()?))
    }
}

impl Parse for ast::FunctionType {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        parser.expect(&KeywordKind::Fn)?;
        let mut content = parser.delim(Delimiter::Paren)?;

        let return_type = if parser.is(&SymbolKind::Arrow) {
            parser.next();
            parser.parse::<ast::Type>()?
        } else {
            ast::Type::new(ast::TypeKind::Void, parser.span())
        };

        Ok(ast::FunctionType {
            arguments: content.parse_comma_separated::<ast::Type>()?,
            return_type: Box::new(return_type),
        })
    }
}

impl Parse for ast::TupleType {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let mut content = parser.delim(Delimiter::Paren)?;
        let elements = content.parse_comma_separated::<ast::Type>()?;
        Ok(ast::TupleType::new(elements))
    }
}

impl Parse for ast::PathType {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        Ok(ast::PathType::new(parser.parse::<ast::Path>()?))
    }
}

impl Parse for ast::TypeKind {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        // parse inferred type
        if parser.is_blank_ident() {
            parser.next();
            Ok(ast::TypeKind::Inferred)

        // parse void type
        } else if parser.is(&KeywordKind::Void) {
            parser.next();
            Ok(ast::TypeKind::Void)

        // parse bool type
        } else if parser.is(&KeywordKind::Bool) {
            parser.next();
            Ok(ast::TypeKind::Bool)

        // parse integer type
        } else if let Some(ty) = parser.try_parse() {
            Ok(ast::TypeKind::Int(ty))

        // parse float type
        } else if let Some(ty) = parser.try_parse() {
            Ok(ast::TypeKind::Float(ty))

        // parse pointer type
        } else if let Some(ty) = parser.try_parse() {
            Ok(ast::TypeKind::Pointer(ty))

        // parse slice type
        } else if let Some(ty) = parser.try_parse() {
            Ok(ast::TypeKind::Slice(ty))

        // parse function type
        } else if let Some(ty) = parser.try_parse() {
            Ok(ast::TypeKind::Function(ty))

        // parse tuple type
        } else if let Some(ty) = parser.try_parse() {
            Ok(ast::TypeKind::Tuple(ty))

        // parse path type
        } else if let Some(ty) = parser.try_parse() {
            Ok(ast::TypeKind::Path(ty))
        } else {
            return Err(parser.expected("type"));
        }
    }
}

impl Parse for ast::Type {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let (kind, span) = parser.parse_spanned()?;
        Ok(ast::Type { kind, span })
    }
}

#[cfg(test)]
mod tests {
    use ritec_span::{Ident, Span};

    use super::*;

    use crate::{ParseBuffer, TokenStream};

    use std::str::FromStr;

    #[test]
    fn parse_int_type() {
        macro_rules! parse_int {
            { $($source:expr => $ty:expr),* $(,)? } => {
                $(
                    let tokens = TokenStream::from_str($source).unwrap();
                    let mut parser = ParseBuffer::new(&tokens);
                    let ty = parser.parse::<ast::IntType>().unwrap();
                    assert_eq!(ty, $ty);
                    assert!(parser.is_empty());
                )*
            };
        }

        parse_int! {
            "i8" => ast::IntType::I8,
            "i16" => ast::IntType::I16,
            "i32" => ast::IntType::I32,
            "i64" => ast::IntType::I64,
            "i128" => ast::IntType::I128,
            "isize" => ast::IntType::Isize,
            "u8" => ast::IntType::U8,
            "u16" => ast::IntType::U16,
            "u32" => ast::IntType::U32,
            "u64" => ast::IntType::U64,
            "u128" => ast::IntType::U128,
            "usize" => ast::IntType::Usize,
        }
    }

    #[test]
    fn parse_float_type() {
        macro_rules! parse_float {
            { $($source:expr => $ty:expr),* $(,)? } => {
                $(
                    let tokens = TokenStream::from_str($source).unwrap();
                    let mut parser = ParseBuffer::new(&tokens);
                    let ty = parser.parse::<ast::FloatType>().unwrap();
                    assert_eq!(ty, $ty);
                    assert!(parser.is_empty());
                )*
            };
        }

        parse_float! {
            "f16" => ast::FloatType::F16,
            "f32" => ast::FloatType::F32,
            "f64" => ast::FloatType::F64,
        }
    }

    #[test]
    fn parse_pointer_type() {
        let tokens = TokenStream::from_str("*i32").unwrap();
        let mut parser = ParseBuffer::new(&tokens);
        let ty = parser.parse::<ast::PointerType>().unwrap();
        assert_eq!(
            ty,
            ast::PointerType::new(ast::Type::new(
                ast::TypeKind::Int(ast::IntType::I32),
                Span::DUMMY
            ))
        );
        assert!(parser.is_empty());
    }

    #[test]
    fn parse_slice_type() {
        let tokens = TokenStream::from_str("[i32]").unwrap();
        let mut parser = ParseBuffer::new(&tokens);
        let ty = parser.parse::<ast::SliceType>().unwrap();
        assert_eq!(
            ty,
            ast::SliceType::new(ast::Type::new(
                ast::TypeKind::Int(ast::IntType::I32),
                Span::DUMMY
            ))
        );
        assert!(parser.is_empty());
    }

    #[test]
    fn parse_function_type() {
        let tokens = TokenStream::from_str("fn(i32, i32) -> i32").unwrap();
        let mut parser = ParseBuffer::new(&tokens);
        let ty = parser.parse::<ast::FunctionType>().unwrap();
        assert_eq!(
            ty,
            ast::FunctionType {
                arguments: vec![
                    ast::Type::new(ast::TypeKind::Int(ast::IntType::I32), Span::DUMMY),
                    ast::Type::new(ast::TypeKind::Int(ast::IntType::I32), Span::DUMMY),
                ],
                return_type: Box::new(ast::Type::new(
                    ast::TypeKind::Int(ast::IntType::I32),
                    Span::DUMMY
                )),
            }
        );
        assert!(parser.is_empty());
    }

    #[test]
    fn parse_tuple_type() {
        let tokens = TokenStream::from_str("(i32, i32)").unwrap();
        let mut parser = ParseBuffer::new(&tokens);
        let ty = parser.parse::<ast::TupleType>().unwrap();
        assert_eq!(
            ty,
            ast::TupleType::new(vec![
                ast::Type::new(ast::TypeKind::Int(ast::IntType::I32), Span::DUMMY),
                ast::Type::new(ast::TypeKind::Int(ast::IntType::I32), Span::DUMMY),
            ])
        );
        assert!(parser.is_empty());
    }

    #[test]
    fn parse_path_type() {
        let tokens = TokenStream::from_str("::foo<i32>").unwrap();
        let mut parser = ParseBuffer::new(&tokens);
        let ty = parser.parse::<ast::PathType>().unwrap();
        assert_eq!(
            ty,
            ast::PathType::new(ast::Path {
                is_absolute: true,
                segments: vec![ast::PathSegment::Item(ast::ItemSegment {
                    ident: Ident::new("foo", Span::DUMMY),
                    generics: vec![ast::Type::new(
                        ast::TypeKind::Int(ast::IntType::I32),
                        Span::DUMMY
                    )],
                })],
                span: Span::DUMMY,
            })
        );
        assert!(parser.is_empty());
    }
}
