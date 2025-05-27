use super::{Parse, ParseResult, ParseStream};

#[derive(Debug)]
pub struct TransformsRule {}

impl Parse for TransformsRule {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        todo!()
    }
}
