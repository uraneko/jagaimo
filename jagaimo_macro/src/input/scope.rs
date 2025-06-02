use proc_macro2::Span;
use syn::parse::{Parse, ParseStream, Parser, Result as PRes};
use syn::{Ident, Type};

#[derive(Debug)]
pub enum Scope {
    Root,
    Space(Ident),
    SpaceOperation { space: Ident, op: Ident },
    Operation(Ident),
}

#[derive(Debug)]
pub enum AliasScope {
    // a space
    S,
    // an operation
    O,
    // a flag
    F,
}

impl TryFrom<Ident> for AliasScope {
    type Error = syn::Error;

    fn try_from(ident: Ident) -> PRes<Self> {
        match ident {
            i if i == Ident::new("s", Span::call_site()) => Ok(Self::S),
            i if i == Ident::new("o", Span::call_site()) => Ok(Self::O),
            i if i == Ident::new("f", Span::call_site()) => Ok(Self::F),
            _ => Err(syn::Error::new(
                Span::call_site(),
                "AliasScope try_from Ident takes 1 of s, o or f idents",
            )),
        }
    }
}
