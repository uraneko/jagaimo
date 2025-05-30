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

use crate::parse::AliasToken;
use crate::parse::{Aliased, CommandRule};
use crate::parse::{CommandStack, Rules, flags::Flag};

pub mod aliases;
pub mod tokenized_commands;

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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TokenizedCommand {
    space: Option<CommandToken>,
    op: Option<CommandToken>,
    flags: Option<Vec<CommandToken>>,
    param: Option<CommandToken>,
}

impl FromIterator<CommandToken> for TokenizedCommand {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = CommandToken>,
    {
        let mut tc = TokenizedCommand::default();
        let mut iter = iter.into_iter();
        while let Some(token) = iter.next() {
            match token {
                CommandToken::Space { .. } => tc.set_space(token),
                CommandToken::Operation { .. } => tc.set_op(token),
                CommandToken::Flag { .. } => tc.push_flag(token),
                CommandToken::Param(_) => tc.set_param(token),
            }
        }

        tc
    }
}

impl From<Vec<CommandToken>> for TokenizedCommand {
    fn from(seq: Vec<CommandToken>) -> Self {
        seq.into_iter().collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandToken {
    Space { ident: Ident, alias: Option<Ident> },
    Operation { ident: Ident, alias: Option<Ident> },
    Flag { flag: Flag, alias: Option<Ident> },
    Param(Type),
}

impl CommandToken {
    fn is_space(&self) -> bool {
        let Self::Space { .. } = self else {
            return false;
        };

        true
    }
    fn is_op(&self) -> bool {
        let Self::Operation { .. } = self else {
            return false;
        };

        true
    }
    fn is_flag(&self) -> bool {
        let Self::Flag { .. } = self else {
            return false;
        };

        true
    }
    fn is_param(&self) -> bool {
        let Self::Param { .. } = self else {
            return false;
        };

        true
    }
}

impl TokenizedCommand {
    pub fn set_space(&mut self, s: CommandToken) {
        if !s.is_space() {
            return;
        }

        self.space = Some(s);
    }

    pub fn set_op(&mut self, o: CommandToken) {
        if !o.is_op() {
            return;
        }

        self.op = Some(o);
    }

    pub fn push_flag(&mut self, f: CommandToken) {
        if !f.is_flag() {
            return;
        }

        match self.flags {
            None => self.flags = Some(vec![f]),
            Some(ref mut flags) => flags.push(f),
        }
    }

    pub fn set_param(&mut self, p: CommandToken) {
        if !p.is_param() {
            return;
        }

        self.param = Some(p);
    }
}

impl From<(CommandRule, &[Aliased])> for TokenizedCommand {
    fn from(value: (CommandRule, &[Aliased])) -> Self {
        let cr = value.0;
        let als = value.1;
        let (spc, op, flgs) = (
            cr.find_space_alias(als),
            cr.find_op_alias(als),
            cr.find_flags_aliases(als),
        );

        let mut tc = Self::default();

        if spc.is_some() {
            tc.space = Some(CommandToken::Space {
                ident: cr.space().unwrap().clone(),
                alias: spc.map(|a| a.clone()),
            });
        };

        if op.is_some() {
            tc.op = Some(CommandToken::Operation {
                ident: cr.op().unwrap().clone(),
                alias: op.map(|a| a.clone()),
            });
        };

        if let Some(p) = cr.params() {
            tc.param = Some(CommandToken::Param(p.clone()));
        }

        if let Some(mut flgs) = flgs {
            let flags = {
                tc.flags = Some(vec![]);

                tc.flags.as_mut().unwrap()
            };
            while let Some(f) = flgs.next() {
                if let Some(flag) = f {
                    flags.push(flag)
                }
            }
        }

        tc
    }
}
