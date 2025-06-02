use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream, Result as ParseResult};
use syn::{Ident, Token, Type, token::Paren};
use syn::{braced, bracketed, parenthesized};

use quote::ToTokens;

use super::AliasToken;
use super::Aliased;
use super::context::Context;
use super::context::Flag;
use super::extract_scope_items;
use super::scope::Scope;
use crate::process::tokenized_commands::CommandToken;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CommandRule {
    space: Option<Ident>,
    op: Option<Ident>,
    params: Option<Type>,
    flags: Option<Vec<Flag>>,
}

impl CommandRule {
    pub fn space(&self) -> Option<&Ident> {
        self.space.as_ref()
    }

    pub fn op(&self) -> Option<&Ident> {
        self.op.as_ref()
    }

    pub fn params(&self) -> Option<&Type> {
        self.params.as_ref()
    }

    pub fn flags(&self) -> Option<&[Flag]> {
        self.flags.as_ref().map(|v| v.as_slice())
    }

    pub fn set_op(&mut self, i: Ident) {
        self.op = Some(i);
    }

    pub fn find_space_alias<'a>(&self, als: &'a [Aliased]) -> Option<&'a Ident> {
        if let Some(spc) = self.space() {
            return als
                .iter()
                .find(|a| {
                    if let AliasToken::Space(s) = a.token() {
                        s == spc
                    } else {
                        false
                    }
                })
                .map(|a| a.alias());
        }

        None
    }

    pub fn find_op_alias<'a>(&self, als: &'a [Aliased]) -> Option<&'a Ident> {
        if let Some(op) = self.op() {
            return als
                .iter()
                .find(|a| {
                    if let AliasToken::Operation(o) = a.token() {
                        o == op
                    } else {
                        false
                    }
                })
                .map(|a| a.alias());
        }

        None
    }
    pub fn find_flags_aliases(
        &self,
        als: &[Aliased],
    ) -> Option<impl Iterator<Item = Option<CommandToken>>> {
        if let Some(flags) = self.flags() {
            Some(flags.iter().map(|flg| {
                als.iter()
                    .find(|a| {
                        if let AliasToken::Flag(f) = a.token() {
                            f == flg.ident()
                        } else {
                            false
                        }
                    })
                    .map(|a| CommandToken::Flag {
                        flag: flg.clone(),
                        alias: Some(a.alias().clone()),
                    })
            }))
        } else {
            None
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ExpandedCommandRule {
    rules: Vec<CommandRule>,
}

impl ExpandedCommandRule {
    pub fn into_rules(self) -> Vec<CommandRule> {
        self.rules
    }

    pub fn rules(&self) -> &[CommandRule] {
        &self.rules
    }
}

// so many clones and vec allocations
// TODO optimize this
impl Parse for ExpandedCommandRule {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        let _rule_name = Ident::parse(stream)?;

        let content;
        _ = braced!(content in stream);
        // there is some scope

        let spaces = if content.peek(Ident::peek_any) {
            extract_scope_items(&content)?
        } else {
            vec![]
        };

        let ops = if content.peek(Ident::peek_any) {
            extract_scope_items(&content)?
        } else {
            vec![]
        };

        let context;
        _ = bracketed!(context in content);

        let (flags, params) = extract_context_items(&context)?;

        let expanded = expand_scopes(spaces, ops)
            .into_iter()
            .map(|mut c| {
                c.flags = Some(flags.clone());
                c.params = params.clone();

                c
            })
            .collect();

        Ok(ExpandedCommandRule { rules: expanded })
    }
}

fn extract_context_items(s: ParseStream) -> ParseResult<(Vec<Flag>, Option<Type>)> {
    let mut f = vec![];
    let mut p = None;

    while !s.is_empty() {
        // flag
        if s.peek(Ident::peek_any) {
            f.push(Flag::parse(s)?);
        // param
        } else {
            _ = <Token![<]>::parse(s)?;
            p = Type::parse(s).ok();
            _ = <Token![>]>::parse(s)?;
        }
    }

    Ok((f, p))
}

fn expand_scopes(spaces: Vec<Ident>, ops: Vec<Ident>) -> Vec<CommandRule> {
    match [spaces.is_empty(), ops.is_empty()] {
        [true, true] => vec![CommandRule::default()],
        [true, false] => ops
            .into_iter()
            .map(|o| CommandRule {
                op: Some(o),
                ..Default::default()
            })
            .collect(),
        [false, true] => spaces
            .into_iter()
            .map(|s| CommandRule {
                space: Some(s),
                ..Default::default()
            })
            .collect(),
        [false, false] => spaces
            .into_iter()
            .map(|s| {
                ops.iter()
                    .map(|o| CommandRule {
                        space: Some(s.clone()),
                        op: Some(o.clone()),
                        ..Default::default()
                    })
                    .collect::<Vec<CommandRule>>()
            })
            .flatten()
            .collect(),
    }
}

impl std::fmt::Display for CommandRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            format!(
                "SPACE<{}> OPERATION<{}> PARAM<{}> FLAGS<{}>\n",
                self.space
                    .as_ref()
                    .map(|i| i.to_string())
                    .unwrap_or("".into()),
                self.op.as_ref().map(|i| i.to_string()).unwrap_or("".into()),
                self.params
                    .as_ref()
                    .map(|t| format!("{:?}", t.to_token_stream().to_string()))
                    .unwrap_or("".into()),
                self.flags
                    .as_ref()
                    .map(|f| f
                        .into_iter()
                        .map(|f| f.to_string())
                        .fold(String::new(), |acc, f| acc + &f + " "))
                    .unwrap_or("".into()),
            )
        )
    }
}
