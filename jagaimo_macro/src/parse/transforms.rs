use super::{Parse, ParseResult, ParseStream};

#[derive(Debug)]
pub struct TransformRule {}

impl Parse for TransformRule {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        todo!()
    }
}
