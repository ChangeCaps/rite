use ritec_core::Ident;
use ritec_error::Diagnostic;

use crate::ParseBuffer;

pub type ParseStream<'a, 'b> = &'a mut ParseBuffer<'b>;
pub type ParseResult<T> = std::result::Result<T, Diagnostic>;

pub trait Parse: Sized {
    fn parse(parser: ParseStream) -> ParseResult<Self>;
}

impl Parse for Ident {
    fn parse(parser: ParseStream) -> ParseResult<Self> {
        parser.ident()
    }
}
