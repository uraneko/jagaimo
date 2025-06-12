use proc_macro2::Span;
use quote::ToTokens;
use quote::quote;
use syn::Error;
use syn::Result as SRes;
use syn::{Attribute, Expr, ExprLit, Ident, Meta};
use syn::{MetaList, MetaNameValue, Path};

use resolve_calling_crate::ResolveCrate;
use toml::Table;

use crate::read::{OpHelp, ReadTomlHelp, RootHelp, SpaceHelp};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HelpAttrs {
    aliases: Vec<Ident>,
    at: String,
}

impl Default for HelpAttrs {
    fn default() -> Self {
        Self {
            at: String::from("help.toml"),
            aliases: vec![],
        }
    }
}

impl HelpAttrs {
    pub fn new(attrs: Vec<Attribute>) -> SRes<Self> {
        let mut res = Self::default();
        let mut attrs = attrs.into_iter();

        while let Some(a) = attrs.next() {
            let meta = a.meta;

            if let Ok(mnv) = meta.require_name_value() {
                res.at = parse_at(mnv)?;
            }
            if let Ok(ml) = meta.require_list() {
                res.aliases = parse_aliases(ml)?;
            }
        }

        Ok(res)
    }

    // reads the toml file found at self.at
    pub fn read_help(&self) -> Table {
        std::fs::read_to_string(ResolveCrate::new().into_string_suffixed(&self.at))
            .unwrap()
            .parse::<Table>()
            .unwrap()
    }
}

pub fn parse_at(mnv: &MetaNameValue) -> SRes<String> {
    if !mnv.path.is_ident("at") {
        return Err(Error::new(
            Span::call_site(),
            "expected at, got unrecognized attr meta name value",
        ));
    }

    let mut s = mnv.value.to_token_stream().to_string();
    s.remove(s.len() - 1);
    s.remove(0);

    Ok(s)
}

enum Alias {
    Space { a: String, t: String },
    Op { a: String, t: String },
    Flag { a: String, t: String },
}

use syn::parse::{ParseStream, Result as PRes};

pub fn parse_aliases(s: proc_macro2::TokenStream) -> Vec<Alias> {
    let s: ParseStream = <TokenStream as Into<ParseStream>>::into(s);
    let mut v = vec![];
    while s.peek(Ident) {
        v.push(Alias::parse(s)?)
    }
}

impl Parse for Alias {
    fn parse(s: ParseStream) -> PRes<Self> {}
}
