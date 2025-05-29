use proc_macro2::TokenStream as TS2;
use quote::quote;
use syn::Variant;
use syn::parse_str;
use syn::token::Paren;
use syn::{
    Ident, Token, Type, braced, bracketed,
    ext::IdentExt,
    parenthesized,
    parse::{Parse, ParseStream, Result as ParseResult},
};

use quote::ToTokens;

use super::scope::Scope;
use crate::process::{dummy_ident, dummy_type};
use std::mem::discriminant;

#[derive(Debug, Clone, PartialEq)]
pub enum Flag {
    Bool(Ident),
    Parameterized(Ident, Type),
}

impl Flag {
    pub fn to_string(&self) -> String {
        match self {
            Self::Bool(i) => i.to_string(),
            Self::Parameterized(i, t) => format!("{}({:?})", i, t.to_token_stream().to_string()),
        }
    }

    pub fn ident(&self) -> &Ident {
        match self {
            Self::Bool(i) => i,
            Self::Parameterized(i, _) => i,
        }
    }
}

#[derive(Debug, Default)]
pub struct FlagsVec {
    flags: Vec<Flag>,
}

impl Parse for Flag {
    fn parse(s: ParseStream<'_>) -> ParseResult<Self> {
        let ident = Ident::parse(s)?;
        if s.is_empty() {
            return Ok(Self::Bool(ident));
        }

        if s.peek(Paren) {
            let content;
            _ = parenthesized!(content in s);
            let mut ty = Type::parse(&content)?;
            return Ok(Self::Parameterized(ident, ty));
        }
        Ok(Self::Bool(ident))
    }
}

impl FlagsVec {
    fn push(&mut self, f: ParseResult<Flag>) {
        if let Ok(f) = f {
            self.flags.push(f);

            return;
        }

        println!("flag was an error {:?}", f);
    }

    fn flags(self) -> Vec<Flag> {
        self.flags
    }
}

impl Parse for FlagsVec {
    fn parse(s: ParseStream) -> ParseResult<Self> {
        let mut f = FlagsVec::default();
        f.push(Flag::parse(s));

        while !s.is_empty() {
            _ = <Token![,]>::parse(s)?;

            f.push(Flag::parse(s));
        }

        Ok(f)
    }
}

impl Flag {
    pub fn is_bool(&self) -> bool {
        discriminant(self) == discriminant(&Self::Bool(dummy_ident()))
    }

    pub fn is_param(&self) -> bool {
        discriminant(self) == discriminant(&Self::Parameterized(dummy_ident(), dummy_type()))
    }

    pub fn to_field(&self) -> [TS2; 2] {
        match self {
            Self::Bool(i) => [quote! { #i: bool }, quote! {}],
            Self::Parameterized(i, t) => [quote! { #i: #t }, quote! {}],
        }
    }
}
