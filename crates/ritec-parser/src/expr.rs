use ritec_ast as ast;
use ritec_core::{BinOp, BoolLiteral, Literal, NullLiteral, UnaryOp};

use crate::{
    path::parse_generics, Delimiter, KeywordKind, Parse, ParseResult, ParseStream, SymbolKind,
};

impl Parse for ast::ParenExpr {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let span = parser.span();
        let mut contents = parser.delim(Delimiter::Paren)?;
        Ok(ast::ParenExpr {
            expr: Box::new(contents.parse()?),
            span,
        })
    }
}

impl Parse for ast::PathExpr {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let (path, span) = parser.parse_spanned()?;
        Ok(ast::PathExpr { path, span })
    }
}

impl Parse for ast::LiteralExpr {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let span = parser.span();
        let literal = if parser.is(&KeywordKind::Null) {
            parser.next();

            Literal::Null(NullLiteral {
                span: span | parser.span(),
            })
        } else if parser.is(&KeywordKind::True) {
            parser.next();

            Literal::Bool(BoolLiteral {
                value: true,
                span: span | parser.span(),
            })
        } else if parser.is(&KeywordKind::False) {
            parser.next();

            Literal::Bool(BoolLiteral {
                value: false,
                span: span | parser.span(),
            })
        } else {
            parser.parse()?
        };

        Ok(ast::LiteralExpr { literal, span })
    }
}

impl Parse for ast::InitField {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let span = parser.span();
        let name = parser.parse()?;
        parser.expect(&SymbolKind::Colon)?;
        let expr = parser.parse()?;
        Ok(ast::InitField {
            ident: name,
            expr,
            span,
        })
    }
}

impl Parse for ast::ClassInitExpr {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let span = parser.span();
        let path = parser.parse()?;
        let mut contents = parser.delim(Delimiter::Brace)?;
        let fields = contents.parse_comma_separated()?;

        Ok(ast::ClassInitExpr {
            class: path,
            fields,
            span,
        })
    }
}

impl Parse for UnaryOp {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        if parser.is(&SymbolKind::Amp) {
            parser.next();
            Ok(UnaryOp::Ref)
        } else if parser.is(&SymbolKind::Star) {
            parser.next();
            Ok(UnaryOp::Deref)
        } else if parser.is(&SymbolKind::Bang) {
            parser.next();
            Ok(UnaryOp::Not)
        } else if parser.is(&SymbolKind::Minus) {
            parser.next();
            Ok(UnaryOp::Neg)
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

impl Parse for BinOp {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        if parser.is(&SymbolKind::Plus) {
            parser.next();
            Ok(BinOp::Add)
        } else if parser.is(&SymbolKind::Minus) {
            parser.next();
            Ok(BinOp::Sub)
        } else if parser.is(&SymbolKind::Star) {
            parser.next();
            Ok(BinOp::Mul)
        } else if parser.is(&SymbolKind::FSlash) {
            parser.next();
            Ok(BinOp::Div)
        } else if parser.is(&SymbolKind::EqualEqual) {
            parser.next();
            Ok(BinOp::Eq)
        } else if parser.is(&SymbolKind::BangEqual) {
            parser.next();
            Ok(BinOp::Ne)
        } else if parser.is(&SymbolKind::Less) {
            parser.next();
            Ok(BinOp::Lt)
        } else if parser.is(&SymbolKind::LessEqual) {
            parser.next();
            Ok(BinOp::Le)
        } else if parser.is(&SymbolKind::Greater) {
            parser.next();
            Ok(BinOp::Gt)
        } else if parser.is(&SymbolKind::GreaterEqual) {
            parser.next();
            Ok(BinOp::Ge)
        } else {
            Err(parser.expected("binary operator"))
        }
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

impl Parse for ast::BreakExpr {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let span = parser.expect(&KeywordKind::Break)?;
        Ok(ast::BreakExpr { span })
    }
}

impl Parse for ast::BlockExpr {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let (block, span) = parser.parse_spanned::<ast::Block>()?;
        Ok(ast::BlockExpr { block, span })
    }
}

impl Parse for ast::IfExpr {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let span = parser.expect(&KeywordKind::If)?;
        let condition = parser.parse()?;
        let then_block = ast::Expr::Block(parser.parse()?);

        let else_block = if parser.is(&KeywordKind::Else) {
            parser.next();

            if parser.is(&KeywordKind::If) {
                Some(Box::new(ast::Expr::If(parser.parse()?)))
            } else {
                Some(Box::new(ast::Expr::Block(parser.parse()?)))
            }
        } else {
            None
        };

        Ok(ast::IfExpr {
            condition: Box::new(condition),
            then_block: Box::new(then_block),
            else_block,
            span,
        })
    }
}

impl Parse for ast::LoopExpr {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        parser.expect(&KeywordKind::Loop)?;
        let (block, span) = parser.parse_spanned()?;
        Ok(ast::LoopExpr { block, span })
    }
}

impl Parse for ast::WhileExpr {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let span = parser.expect(&KeywordKind::While)?;
        let condition = parser.parse()?;
        let block = parser.parse()?;
        Ok(ast::WhileExpr {
            condition: Box::new(condition),
            block,
            span: span | parser.span(),
        })
    }
}

fn parse_term(parser: ParseStream) -> ParseResult<ast::Expr> {
    if let Some(expr) = parser.try_parse() {
        Ok(ast::Expr::ClassInit(expr))
    } else if let Some(expr) = parser.try_parse() {
        Ok(ast::Expr::Path(expr))
    } else if let Some(literal) = parser.try_parse::<ast::LiteralExpr>() {
        Ok(ast::Expr::Literal(literal))
    } else if parser.is(&Delimiter::Paren) {
        Ok(ast::Expr::Paren(parser.parse()?))
    } else {
        Err(parser.expected("expression"))
    }
}

fn parse_field(parser: ParseStream) -> ParseResult<ast::Expr> {
    let span = parser.span();
    let mut base = parse_term(parser)?;

    while parser.is(&SymbolKind::Dot) {
        parser.next();
        let field = parser.parse()?;
        let generics = parse_generics(parser);

        base = ast::Expr::Field(ast::FieldExpr {
            class: Box::new(base),
            field,
            generics,
            span: span | parser.span(),
        });
    }

    Ok(base)
}

fn parse_call(parser: ParseStream) -> ParseResult<ast::Expr> {
    let span = parser.span();
    let callee = parse_field(parser)?;

    if let Ok(mut contents) = parser.delim(Delimiter::Paren) {
        let arguments = contents.parse_comma_separated()?;

        Ok(ast::Expr::Call(ast::CallExpr {
            callee: Box::new(callee),
            arguments,
            span: span | parser.span(),
        }))
    } else {
        Ok(callee)
    }
}

fn parse_unary(parser: ParseStream) -> ParseResult<ast::Expr> {
    if let Some(operator) = parser.try_parse::<UnaryOp>() {
        Ok(ast::Expr::Unary(ast::UnaryExpr {
            operator,
            operand: Box::new(parse_unary(parser)?),
            span: parser.span(),
        }))
    } else {
        parse_call(parser)
    }
}

fn parse_as(parser: ParseStream) -> ParseResult<ast::Expr> {
    let mut expr = parse_unary(parser)?;

    while parser.is(&KeywordKind::As) {
        parser.next();
        let ty = parser.parse()?;
        expr = ast::Expr::As(ast::AsExpr {
            expr: Box::new(expr),
            ty,
            span: parser.span(),
        });
    }

    Ok(expr)
}

fn parse_binary(parser: ParseStream) -> ParseResult<ast::Expr> {
    let lhs = parse_as(parser)?;

    if let Some(operator) = parser.try_parse::<BinOp>() {
        let rhs = parse_binary(parser)?;

        if let ast::Expr::Binary(ref rhs) = rhs {
            if rhs.operator.precedence() <= operator.precedence() {
                return Ok(ast::Expr::Binary(ast::BinaryExpr {
                    lhs: Box::new(ast::Expr::Binary(ast::BinaryExpr {
                        lhs: Box::new(lhs),
                        operator,
                        rhs: rhs.lhs.clone(),
                        span: parser.span(),
                    })),
                    operator: rhs.operator,
                    rhs: rhs.rhs.clone(),
                    span: parser.span(),
                }));
            }
        }

        Ok(ast::Expr::Binary(ast::BinaryExpr {
            operator,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            span: parser.span(),
        }))
    } else {
        Ok(lhs)
    }
}

fn parse_assign(parser: ParseStream) -> ParseResult<ast::Expr> {
    let span = parser.span();
    let expr = parse_binary(parser)?;

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
        } else if parser.is(&KeywordKind::Break) {
            Ok(ast::Expr::Break(parser.parse()?))
        } else if parser.is(&Delimiter::Brace) {
            Ok(ast::Expr::Block(parser.parse()?))
        } else if parser.is(&KeywordKind::If) {
            Ok(ast::Expr::If(parser.parse()?))
        } else if parser.is(&KeywordKind::Loop) {
            Ok(ast::Expr::Loop(parser.parse()?))
        } else if parser.is(&KeywordKind::While) {
            Ok(ast::Expr::While(parser.parse()?))
        } else {
            parse_assign(parser)
        }
    }
}
