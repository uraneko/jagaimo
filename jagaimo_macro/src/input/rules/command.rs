use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use proc_macro2::Span;
use quote::ToTokens;
use syn::Token;
use syn::parse::{Parse, ParseStream, Result as PRes};
use syn::token::Bracket;
use syn::{Ident, Type};
use syn::{braced, bracketed, parenthesized};

use super::Flag;
use crate::input::scope::{Context, Scope};
use crate::input::to_enforced_ident_nc;

// TODO
// deduplication of rules
#[derive(Debug, Clone, Eq)]
pub struct CommandRule {
    space: Ident,
    bare_space: bool,
    op: Ident,
    bare_op: bool,
    flags: Option<Vec<Flag>>,
    params: Option<Type>,
}

// WARN 2 command rules that have the same scope (space and op)
// should be considered equal reagrdless of their context

impl Hash for CommandRule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.space.hash(state);
        self.op.hash(state);
    }
}

impl PartialEq for CommandRule {
    fn eq(&self, other: &Self) -> bool {
        self.space == other.space && self.op == other.op
    }
}

impl CommandRule {
    pub fn contains_space(&self) -> bool {
        !self.bare_space
    }

    pub fn contains_op(&self) -> bool {
        !self.bare_op
    }

    pub fn contains_flags(&self) -> bool {
        self.flags.is_some()
    }
}

impl CommandRule {
    pub fn space_cloned(&self) -> Ident {
        self.space.clone()
    }

    pub fn op_cloned(&self) -> Ident {
        self.op.clone()
    }

    pub fn space(&self) -> &Ident {
        &self.space
    }

    pub fn op(&self) -> &Ident {
        &self.op
    }

    pub fn scope_hash(&self) -> u64 {
        <&Self as Into<Scope>>::into(&self).into_hash()
    }

    pub fn context(&self) -> Context {
        self.into()
    }

    pub fn prefix_op(&mut self) {
        self.op = Ident::new(
            &{
                let s = format!("{}{}", self.space, self.op);

                s
            },
            Span::call_site(),
        );
    }

    pub fn flags(&self) -> Option<&[Flag]> {
        self.flags.as_ref().map(|flags| flags.as_slice())
    }

    pub fn flags_idents(&self) -> Option<Vec<&Ident>> {
        self.flags
            .as_ref()
            .map(|f| f.into_iter().map(|f| f.ident()).collect())
    }

    pub fn params(&self) -> Option<&Type> {
        self.params.as_ref()
    }
}

#[derive(Debug)]
pub struct ExpandingCommandRule {
    spaces: Vec<Ident>,
    ops: Vec<Ident>,
    bare_op: bool,
    bare_space: bool,
    flags: Vec<Flag>,
    params: Option<Type>,
}

fn extract_context_tokens(s: ParseStream) -> PRes<(Vec<Flag>, Option<Type>)> {
    let mut f = HashSet::new();
    let p = if s.peek(Token![<]) {
        _ = <Token![<]>::parse(s)?;
        let p = Type::parse(s)?;
        _ = <Token![>]>::parse(s)?;

        Some(p)
    } else {
        None
    };

    while !s.is_empty() {
        f.insert(Flag::parse(s)?);
    }

    Ok((f.into_iter().collect(), p))
}

pub fn extract_scope_tokens(s: ParseStream) -> PRes<Vec<Ident>> {
    // anonymous scope
    if s.peek(Bracket) {
        return Ok(vec![]);
    }
    // just because next is not a [
    // doesnt mean it would be an ident
    // but failing is such a case is intended behaviour
    let _i = Ident::parse(&s)?;

    let scopes;
    _ = parenthesized!(scopes in s);
    // use punctuated instead
    scopes
        .parse_terminated(Ident::parse, Token![,])
        .map(|p| p.into_iter().collect())
}

// makes a new ident for a bare operation
fn bare_ident(ident: &Ident) -> Ident {
    Ident::new(&(ident.to_string() + "Bare"), Span::call_site())
}

fn bare_str_ident(s: &str) -> Ident {
    Ident::new(&(s.to_string() + "Bare"), Span::call_site())
}

impl ExpandingCommandRule {
    fn new(spaces: Vec<Ident>, ops: Vec<Ident>, flags: Vec<Flag>, params: Option<Type>) -> Self {
        Self {
            bare_space: spaces.is_empty(),
            bare_op: ops.is_empty(),
            spaces,
            ops,
            flags,
            params,
        }
    }

    fn take_flags(&mut self) -> Option<Vec<Flag>> {
        if self.flags.is_empty() {
            None
        } else {
            Some(std::mem::take(&mut self.flags))
        }
    }

    fn clone_flags(&self) -> Option<Vec<Flag>> {
        if self.flags.is_empty() {
            None
        } else {
            Some(self.flags.clone())
        }
    }

    fn clone_params(&self) -> Option<Type> {
        self.params.clone()
    }

    pub fn resolve_naming_conventions(&mut self, ignore_nc: bool) {
        if ignore_nc {
            return;
        }

        self.spaces.iter_mut().for_each(|s| {
            to_enforced_ident_nc(s);
        });
        self.ops.iter_mut().for_each(|s| {
            to_enforced_ident_nc(s);
        });
    }

    pub fn resolve_bare_scopes(&mut self, root_name: &str) {
        match [self.spaces.is_empty(), self.ops.is_empty()] {
            [false, false] => (),
            [true, true] => {
                let ident = bare_str_ident(root_name);
                self.spaces.push(ident.clone());
                self.ops.push(ident);
            }
            [true, false] => {
                self.spaces.push(Ident::new(root_name, Span::call_site()));
            }
            [false, true] => {
                self.spaces.iter().for_each(|s| {
                    self.ops.push(bare_ident(s));
                });
            }
        }
    }

    pub fn expand(mut self) -> Vec<CommandRule> {
        match [self.bare_space, self.bare_op] {
            [true, true] => vec![CommandRule {
                flags: self.take_flags(),
                params: self.params,
                bare_space: self.bare_space,
                bare_op: self.bare_op,
                space: self.spaces.remove(0),
                op: self.ops.remove(0),
            }],

            [true, false] => {
                let ops = std::mem::take(&mut self.ops);
                ops.into_iter()
                    .map(|o| CommandRule {
                        op: o,
                        space: self.spaces[0].clone(),
                        bare_op: self.bare_op,
                        bare_space: self.bare_space,
                        flags: self.clone_flags(),
                        params: self.clone_params(),
                    })
                    .collect()
            }

            [false, true] => {
                let spaces = std::mem::take(&mut self.spaces);
                spaces
                    .into_iter()
                    .map(|s| CommandRule {
                        op: self.ops[0].clone(),
                        space: s,
                        bare_op: self.bare_op,
                        bare_space: self.bare_space,
                        params: self.clone_params(),
                        flags: self.clone_flags(),
                    })
                    .collect()
            }

            [false, false] => self
                .spaces
                .iter()
                .map(|s| {
                    self.ops.iter().map(|o| CommandRule {
                        space: s.clone(),
                        op: o.clone(),
                        bare_space: self.bare_space,
                        bare_op: self.bare_op,
                        flags: self.clone_flags(),
                        params: self.clone_params(),
                    })
                })
                .flatten()
                .collect(),
        }
    }
}

impl Parse for ExpandingCommandRule {
    fn parse(stream: ParseStream) -> PRes<Self> {
        let _rule_name = Ident::parse(stream)?;

        let content;
        _ = braced!(content in stream);
        // there is some scope

        let spaces = extract_scope_tokens(&content)?;
        let ops = extract_scope_tokens(&content)?;

        let context;
        _ = bracketed!(context in content);
        let (flags, params) = extract_context_tokens(&context)?;

        Ok(ExpandingCommandRule::new(spaces, ops, flags, params))
    }
}

impl std::fmt::Display for CommandRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SPACE<{}> OPERATION<{}> PARAM<{}> FLAGS<{}>\n",
            self.space.to_string(),
            self.op.to_string(),
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
    }
}
