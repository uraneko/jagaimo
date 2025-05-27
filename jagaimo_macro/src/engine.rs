use std::mem::discriminant;

pub use proc_macro2::Span;
pub use syn::ext::IdentExt;
pub use syn::parse::{Parse, ParseStream, Result as ParseResult};
pub use syn::{Ident, Lit, Token, Type};
pub use syn::{braced, bracketed, parenthesized, parse_str};

pub mod parse;
pub mod process;

pub use parse::{
    CommandTree, FlagsRule, OperationsRule, ParamsRule, RealmsRule, RuleBook, flags::Flag,
};

pub fn dummy_ident() -> Ident {
    Ident::new("dummy", Span::call_site())
}

pub fn dummy_type() -> Type {
    parse_str::<Type>("()").unwrap()
}

pub fn capitalize_ident(i: &Ident) -> Ident {
    Ident::new(
        &i.to_string()
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if i == 0 {
                    char::to_ascii_uppercase(&c)
                } else {
                    c
                }
            })
            .collect::<String>(),
        Span::call_site(),
    )
}

pub fn capitalize_ident_to_string(i: &Ident) -> String {
    i.to_string()
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if i == 0 {
                char::to_ascii_uppercase(&c)
            } else {
                c
            }
        })
        .collect::<String>()
}
