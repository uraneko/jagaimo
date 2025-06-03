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
use crate::output::alias_generation::AliasGenerator;
use crate::output::commands_tokenizer::{AliasLookup, TokenizedCommand};

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

impl Parse for Rules {
    fn parse(stream: ParseStream) -> PRes<Self> {
        let mut alias = vec![];
        let mut cmd = vec![];
        let mut trnsf = vec![];
        while !stream.is_empty() {
            // TODO this could be better handled
            match stream.fork().parse::<Ident>()? {
                i if i == Ident::new("c", Span::call_site()) => {
                    cmd.extend(ExpandedCommandRule::parse(stream)?.0)
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
                        "expected c, t, s, o or f ident",
                    ));
                }
            }
        }

        Ok(Self { alias, cmd, trnsf })
    }
}

#[derive(Debug, Clone, Eq)]
pub struct AliasRule {
    scoped: AliasScope,
    token: Ident,
    alias: Ident,
}

impl Hash for AliasRule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.scoped.hash(state);
        self.token.hash(state);
    }
}

impl PartialEq for AliasRule {
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token && self.scoped == other.scoped
    }
}

impl AliasRule {
    pub fn new(scoped: AliasScope, token: Ident, alias: Ident) -> Self {
        Self {
            scoped,
            token,
            alias,
        }
    }

    pub fn scope(&self) -> &AliasScope {
        &self.scoped
    }

    pub fn token(&self) -> &Ident {
        &self.token
    }

    pub fn alias(&self) -> &Ident {
        &self.alias
    }
}

impl Parse for AliasRule {
    fn parse(stream: ParseStream) -> PRes<Self> {
        let content;
        let scoped = Ident::parse(stream)?.try_into()?;
        _ = parenthesized!(content in stream);
        let token = Ident::parse(&content)?;
        _ = <Token![=]>::parse(&stream)?;
        let alias = Ident::parse(&stream)?;

        Ok(Self {
            token,
            alias,
            scoped,
        })
    }
}

impl std::fmt::Display for AliasRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}:{} -> {}", self.scope(), self.token(), self.alias)
    }
}

#[derive(Debug, Default)]
pub struct TransformRule {}

impl Parse for TransformRule {
    fn parse(stream: ParseStream) -> PRes<Self> {
        todo!()
    }
}

// TODO
// deduplication of rules
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CommandRule {
    space: Option<Ident>,
    op: Option<Ident>,
    flags: Option<Vec<Flag>>,
    params: Option<Type>,
}

impl CommandRule {
    pub fn contains_space(&self) -> bool {
        self.space.is_some()
    }

    pub fn contains_op(&self) -> bool {
        self.op.is_some()
    }

    pub fn contains_flags(&self) -> bool {
        self.flags.is_some()
    }
}

impl CommandRule {
    pub fn space(&self) -> Option<&Ident> {
        self.space.as_ref()
    }

    pub fn op(&self) -> Option<&Ident> {
        self.op.as_ref()
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
    pub fn space_cloned(&self) -> Option<Ident> {
        self.space.clone()
    }

    pub fn op_cloned(&self) -> Option<Ident> {
        self.op.clone()
    }
}

#[derive(Debug)]
pub struct ExpandedCommandRule(Vec<CommandRule>);

fn extract_context_tokens(s: ParseStream) -> PRes<(Vec<Flag>, Option<Type>)> {
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

impl ExpandedCommandRule {
    fn new(spaces: Vec<Ident>, ops: Vec<Ident>, flags: Vec<Flag>, params: Option<Type>) -> Self {
        Self(match [spaces.is_empty(), ops.is_empty()] {
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
                .map(|mut c| {
                    c.flags = Some(flags.clone());
                    c.params = params.clone();

                    c
                })
                .collect(),
        })
    }
}

impl Parse for ExpandedCommandRule {
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

        Ok(ExpandedCommandRule::new(spaces, ops, flags, params))
    }
}

impl std::fmt::Display for CommandRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
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
    }
}
