use std::collections::HashMap;
use std::mem::discriminant;

pub use proc_macro::TokenStream as TS;
pub use proc_macro2::Span;
pub use proc_macro2::TokenStream as TS2;
use quote::quote;
use syn::Variant;
pub use syn::ext::IdentExt;
pub use syn::parse::{Parse, ParseStream, Result as ParseResult};
pub use syn::{Ident, Lit, Token, Type};
pub use syn::{braced, bracketed, parenthesized, parse_str};

pub use crate::parse::{CommandStack, Rules, flags::Flag};

pub mod aliases;

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

pub fn ident_to_variant(i: &Ident) -> Variant {
    parse_str::<Variant>(&format!("{}({})", i, i)).unwrap()
}

#[derive(Debug, Clone, PartialEq)]
struct TokenizedCommand {
    seq: Vec<CommandToken>,
}

use crate::parse::AliasToken;

impl From<Vec<CommandToken>> for TokenizedCommand {
    fn from(seq: Vec<CommandToken>) -> Self {
        Self { seq }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct TokenizedCommands {
    commands: Vec<TokenizedCommand>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandToken {
    Space { ident: Ident, alias: Option<Ident> },
    Operation { ident: Ident, alias: Option<Ident> },
    Flag { flag: Flag, alias: Option<Ident> },
    Param(Type),
}
