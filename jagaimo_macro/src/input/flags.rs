use quote::ToTokens;
use syn::Token;
use syn::parse::{Parse, ParseStream, Parser, Result as PRes};
use syn::{Ident, Type};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Flag {
    Bool(Ident),
    Parameterized { ident: Ident, ty: Type },
}

impl Flag {
    pub fn ident(&self) -> &Ident {
        match self {
            Self::Bool(i) => i,
            Self::Parameterized { ident, .. } => ident,
        }
    }
}

impl Parse for Flag {
    fn parse(s: ParseStream<'_>) -> PRes<Self> {
        let ident = Ident::parse(s)?;
        if s.is_empty() {
            return Ok(Self::Bool(ident));
        }

        if s.peek(Token![<]) {
            // _ = parenthesized!(content in s);
            _ = <Token![<]>::parse(s)?;
            let ty = Type::parse(s)?;
            _ = <Token![>]>::parse(s)?;

            return Ok(Self::Parameterized { ident, ty });
        }

        Ok(Self::Bool(ident))
    }
}

impl std::fmt::Display for Flag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Bool(i) => format!("{}", i),
                Self::Parameterized { ident, ty } =>
                    format!("{}<{}>", ident, ty.to_token_stream().to_string()),
            }
        )
    }
}
