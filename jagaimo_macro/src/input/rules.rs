use proc_macro2::Span;
use quote::ToTokens;
use syn::Token;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream, Result as PRes};
use syn::token::Bracket;
use syn::{Ident, Type};
use syn::{braced, bracketed, parenthesized};

use super::{AliasScope, Flag};

#[derive(Debug, Default)]
pub struct Rules {
    alias: Vec<AliasRule>,
    cmd: Vec<CommandRule>,
    trnsf: Vec<TransformRule>,
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

#[derive(Debug)]
pub struct AliasRule {
    scoped: AliasScope,
    token: Ident,
    alias: Ident,
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

#[derive(Debug, Default)]
pub struct TransformRule {}

impl Parse for TransformRule {
    fn parse(stream: ParseStream) -> PRes<Self> {
        todo!()
    }
}

#[derive(Debug, Default)]
pub struct CommandRule {
    space: Option<Ident>,
    op: Option<Ident>,
    flags: Option<Vec<Flag>>,
    params: Option<Type>,
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
