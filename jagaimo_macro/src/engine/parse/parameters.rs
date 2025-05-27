use super::{
    Ident, IdentExt, Parse, ParseResult, ParseStream, Scope, Token, Type, braced, bracketed,
    parenthesized,
};

#[derive(Debug)]
pub struct ParamsRule {
    scope: Scope,
    params: Vec<Type>,
}

impl Parse for ParamsRule {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        let mut content;
        let brace = braced!(content in stream);

        let mut idents: Vec<Ident> = vec![];
        let mut scope = vec![];
        let mut temp;
        while content.peek(Ident::peek_any) {
            scope.extend([Ident::parse(&content)?, {
                let _paren = parenthesized!(temp in content);

                Ident::parse(&temp)?
            }]);
        }
        let bracket = bracketed!(content in content);
        let arr = content
            .parse_terminated(Type::parse, Token![,])?
            .into_iter()
            .map(|v| v)
            .collect::<Vec<Type>>();

        Ok(Self {
            scope: scope.into(),
            params: arr,
        })
    }
}

impl ParamsRule {
    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn params(&self) -> &[Type] {
        &self.params
    }
}
