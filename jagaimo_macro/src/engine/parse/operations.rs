use super::{Ident, Parse, ParseResult, ParseStream, Scope, Token, braced, bracketed};

#[derive(Debug)]
pub struct OperationsRule {
    scope: Scope,
    ops: Vec<Ident>,
}

impl Parse for OperationsRule {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        let mut content;
        let brace = braced!(content in stream);
        let scope = Ident::parse(&content);
        let scope = if scope.is_err() {
            Scope::Executable
        } else {
            Scope::Realm(scope?)
        };

        let bracket = bracketed!(content in content);
        let arr = content
            .parse_terminated(Ident::parse, Token![,])?
            .into_iter()
            .collect::<Vec<Ident>>();

        Ok(Self { scope, ops: arr })
    }
}

impl OperationsRule {
    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn ops(&self) -> &[Ident] {
        &self.ops
    }
}
