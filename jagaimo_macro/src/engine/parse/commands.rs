use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream, Result as ParseResult};
use syn::{Ident, Token, Type, token::Paren};
use syn::{braced, bracketed, parenthesized};

use quote::ToTokens;

use super::context::Context;
use super::context::Flag;
use super::extract_scope_items;
use super::scope::Scope;

#[derive(Debug, PartialEq, Default)]
pub struct CommandRule {
    space: Option<Ident>,
    op: Option<Ident>,
    params: Option<Type>,
    flags: Option<Vec<Flag>>,
}

#[derive(Debug, Default, PartialEq)]
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

    if s.peek(Ident::peek_any) {
        f.push(Flag::parse(s)?);
    // param
    } else {
        let ty;
        _ = parenthesized!(ty in s);
        p = Type::parse(&ty).ok();
    }

    while !s.is_empty() {
        _ = <Token![,]>::parse(s)?;
        // flag
        if s.peek(Ident::peek_any) {
            f.push(Flag::parse(s)?);
        // param
        } else {
            let ty;
            _ = parenthesized!(ty in s);
            p = Type::parse(&ty).ok();
        }
    }
    println!("p {:?}", p);
    println!("f {:?}", f);

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
