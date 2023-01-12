use ritec_ast as ast;

use crate::{Delimiter, Parse, ParseResult, ParseStream};

impl Parse for ast::Block {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        let span = parser.span();
        let mut content = parser.delim(Delimiter::Brace)?;

        // parse statements in the block
        let mut stmts = Vec::new();
        while !content.is_empty() {
            stmts.push(content.parse()?);
        }

        Ok(ast::Block {
            stmts,
            span: span | parser.span(),
        })
    }
}
