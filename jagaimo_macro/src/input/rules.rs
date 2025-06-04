use std::collections::HashSet;

use proc_macro2::Span;
use syn::Ident;
use syn::parse::{Parse, ParseStream, Result as PRes};

use super::{AliasScope, Flag};
use crate::output::AliasGenerator;
use crate::output::{AliasLookup, TokenizedCommand};

pub mod alias;
pub mod command;
pub mod transform;

pub use alias::AliasRule;
pub use command::{CommandRule, ExpandingCommandRule};
pub use transform::TransformRule;

#[derive(Debug, Default)]
pub struct RulesUnresolved {
    alias: Vec<AliasRule>,
    cmd: Vec<ExpandingCommandRule>,
    trnsf: Vec<TransformRule>,
}

impl RulesUnresolved {
    pub fn nameless_resolution(self, root_name: &str) -> Rules {
        Rules {
            alias: self.alias,
            trnsf: self.trnsf,
            // WARN too much collect/clone between this bit of code and
            // the expand method
            cmd: self
                .cmd
                .into_iter()
                .map(|exp| exp.expand(root_name))
                .flatten()
                .collect::<HashSet<CommandRule>>()
                .into_iter()
                .collect(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Rules {
    alias: Vec<AliasRule>,
    cmd: Vec<CommandRule>,
    trnsf: Vec<TransformRule>,
}

impl Rules {
    pub fn cmd_ref(&self) -> &[CommandRule] {
        &self.cmd
    }

    pub fn cmd_mut(&mut self) -> &mut Vec<CommandRule> {
        &mut self.cmd
    }

    pub fn alias_ref(&self) -> &[AliasRule] {
        &self.alias
    }

    pub fn alias_mut(&mut self) -> &mut Vec<AliasRule> {
        &mut self.alias
    }
}

impl Rules {
    pub fn alias_generator(&mut self, auto_alias: bool) {
        if !auto_alias {
            return;
        }

        let mut alias = std::mem::take(self.alias_mut());
        let mut alias_gen = AliasGenerator::new(&mut alias, self.cmd_ref());
        alias_gen.generate_aliases();

        self.alias = std::mem::take(&mut alias);
    }
}

impl Rules {
    pub fn cmds_tokenizer(&self) -> Vec<TokenizedCommand> {
        self.cmd
            .iter()
            .map(|cmd| AliasLookup::new(cmd, &self.alias))
            .map(|al| al.lookup())
            .collect()
    }
}

impl Parse for RulesUnresolved {
    fn parse(stream: ParseStream) -> PRes<Self> {
        let mut alias = vec![];
        let mut cmd = vec![];
        let mut trnsf = vec![];
        while !stream.is_empty() {
            // TODO this could be better handled
            match stream.fork().parse::<Ident>()? {
                i if i == Ident::new("c", Span::call_site()) => {
                    cmd.push(ExpandingCommandRule::parse(stream)?)
                }

                i if i == Ident::new("t", Span::call_site()) => {
                    unimplemented!("transform rules have not been implemented yet")
                }
                i if [
                    Ident::new("s", Span::call_site()),
                    Ident::new("o", Span::call_site()),
                    Ident::new("f", Span::call_site()),
                ]
                .contains(&i) =>
                {
                    alias.push(AliasRule::parse(stream)?)
                }
                val => {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        format!("expected c, t, s, o or f ident, got {}", val),
                    ));
                }
            }
        }

        Ok(Self { alias, cmd, trnsf })
    }
}
