use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use proc_macro2::Span;
use quote::ToTokens;
use syn::Token;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream, Result as PRes};
use syn::token::Bracket;
use syn::{Ident, Type};
use syn::{braced, bracketed, parenthesized};

use super::{AliasScope, Flag};

// TODO
// deduplication of rules
#[derive(Debug, Clone, Eq)]
pub struct CommandRule {
    space: Ident,
    nameless_space: bool,
    op: Ident,
    nameless_op: bool,
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
        self.nameless_space
    }

    pub fn contains_op(&self) -> bool {
        self.nameless_op
    }

    pub fn contains_flags(&self) -> bool {
        self.flags.is_some()
    }
}

impl CommandRule {
    pub fn space(&self) -> &Ident {
        &self.space
    }

    pub fn op(&self) -> &Ident {
        &self.op
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

impl CommandRule {
    pub fn space_cloned(&self) -> Ident {
        self.space.clone()
    }

    pub fn op_cloned(&self) -> Ident {
        self.op.clone()
    }
}

#[derive(Debug)]
pub struct ExpandingCommandRule {
    spaces: Vec<Ident>,
    ops: Vec<Ident>,
    flags: Vec<Flag>,
    params: Option<Type>,
}

// TODO make ExpandingCommandRule take a hashset
// and impl hash and partial eq for UnresolvedCommnadRule
// comparing only space and operation equality
// this will give us command rule dedup

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

// makes a new ident for a nameless operation
fn nameless_ident(ident: &Ident) -> Ident {
    Ident::new(&(ident.to_string() + "Nameless"), Span::call_site())
}

fn nameless_str_ident(s: &str) -> Ident {
    Ident::new(&(s.to_string() + "NameLess"), Span::call_site())
}

impl ExpandingCommandRule {
    fn new(spaces: Vec<Ident>, ops: Vec<Ident>, flags: Vec<Flag>, params: Option<Type>) -> Self {
        Self {
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

    pub fn expand(mut self, root_name: &str) -> Vec<CommandRule> {
        match [self.spaces.is_empty(), self.ops.is_empty()] {
            [true, true] => vec![CommandRule {
                flags: self.take_flags(),
                params: self.params,
                nameless_space: true,
                nameless_op: true,
                space: Ident::new(root_name, Span::call_site()),
                op: nameless_str_ident(root_name),
            }],
            [true, false] => {
                let ops = std::mem::take(&mut self.ops);
                ops.into_iter()
                    .map(|o| CommandRule {
                        op: o,
                        space: Ident::new(root_name, Span::call_site()),
                        nameless_op: false,
                        nameless_space: true,
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
                        op: nameless_ident(&s),
                        space: s,
                        nameless_op: true,
                        nameless_space: false,
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
                        nameless_space: false,
                        nameless_op: false,
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
