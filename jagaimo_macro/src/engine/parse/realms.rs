use super::{Ident, Parse, ParseResult, ParseStream, Token, braced, bracketed};

#[derive(Debug, Default)]
pub struct RealmsRule {
    realms: Vec<Ident>,
}

impl Parse for RealmsRule {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        let mut content;
        let brace = braced!(content in stream);
        let bracket = bracketed!(content in content);
        let arr = content
            .parse_terminated(Ident::parse, Token![,])?
            .into_iter()
            .collect::<Vec<Ident>>();

        Ok(Self { realms: arr })
    }
}

impl RealmsRule {
    pub fn realms(&self) -> &[Ident] {
        &self.realms
    }
}
