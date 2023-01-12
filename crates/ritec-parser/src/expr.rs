use ritec_ast as ast;

use crate::{Parse, ParseResult, ParseStream};

impl Parse for ast::PathExpr {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        Ok(ast::PathExpr {
            path: parser.parse()?,
        })
    }
}

impl Parse for ast::Expr {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        Ok(ast::Expr::Path(parser.parse()?))
    }
}
