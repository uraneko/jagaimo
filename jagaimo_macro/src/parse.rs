//! the engine takes the macro inputs and parses them
//! into a data form (Rules) that can be used by the macro
//! so to say, the macro's direct input is the Rules struct
//!
//! RULES TYPES:
//! * r(egions) => defines all the spaces of the root scope, if any,
//! passing more than 1 such rule to the macro will trigger a panic
//! * o(peration) => defines the posiible operations of a scope
//! * f(lag) => defines the possible flags of a scope
//! * p(arameter) => defines the possible parameter(s) and their types for a given scope(s)
//! * t(ransform) => defines a transformation that takes a token and transforms it into another
//!     * tr(eplace) => takes an input and output token, replace input with output
//!     // TODO this should take a scope also, and maybe a position
//!     * tf(unctional) => takes an input token pattern and a closure that returns another token
//! NOTE specific rules always overwrite more general rules
//! i.e., { r(colls) o(add) [...] } always superseeds { o(add) [...] }
//! for the root -> r(colls) -> o(add) scope
//!
//! SYNTAX FLAGS:
//! * LowerLong(scope(s), ...) => given scopes have to be in lowercase long format
//! * LowerShort(scope(s), ...) => given scopes have to be in lowercase short format
//! * AliasEager(scope(s), ...) => auto generate aliases for the given scope items when
//! possible
//! * Alias(scope(item) = alias) => manual alias, used in the form:
//! Alias(r(collections) = colls)
//! <- collections space can be aliased as colls, parser accepts both
//! <- superseeds auto alias generation
//! Alias(r(collectibles) = colls)
//! <- this rule would be ignored, superseeded by the above defined colls alias
//! <- 2 aliases of the same value can not coexist, only the first one is accepted
//! * AcceptSnake(scope(s), ...) => accept both snake and kebab (default) cases for given scopes
//! * SnakeOnly(scope(s), ...) => only snake (_) case is valid for the given scope(s) items
//!
//! ATTRIBUTES:
//! * no_help <- dont generate a help command
//! * no_version <- dont generate a version command
//! * fish_cmp <- generate completion for fish shell
//! * nu_cmp <- generate completion for nu shell
//! * branch_off_root
//! <- top level scopes are branched out into their own cli tools
//! i.e., if you have a cli tool called `ct` with 4 spaces `a` `b` `c` and `d`
//! thi attribute will generate code for 4 different cli tools `ct-a`, `ct-b`, `ct-c` and `ct-d`
//! instead of generating code for a single cli tool;`ct`
//! * root_name
//! <- renames the resulting top level cli type, default is crate name
//! follows the rust naming convetions
//! * ignore_naming_conventions <- turns off rust naming convetions for cli top level type
use proc_macro2::Span;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream, Result as ParseResult};
use syn::{Ident, Lit, token::Bracket};
use syn::{braced, bracketed, parenthesized};

use std::mem::discriminant;

use crate::process::dummy_ident;
use crate::resolve_crate::ResolveCrate;
use syn::Token;

pub mod commands;
pub mod context;
pub mod flags;
pub mod scope;
pub mod transforms;

pub use commands::{CommandRule, ExpandedCommandRule};
pub use transforms::TransformRule;

pub fn extract_scope_items(s: ParseStream) -> ParseResult<Vec<Ident>> {
    if s.peek(Bracket) {
        return Ok(vec![]);
    }
    let i = Ident::parse(&s)?;

    let scopes;
    _ = parenthesized!(scopes in s);
    // use punctuated instead
    scopes
        .parse_terminated(Ident::parse, Token![,])
        .map(|p| p.into_iter().collect())
}

#[derive(Debug, Default)]
pub struct Rules {
    commands: Vec<CommandRule>,
    transforms: Vec<TransformRule>,
}

impl Rules {
    fn push_commands(&mut self, commands: ExpandedCommandRule) {
        self.commands.extend(commands.into_rules());
    }

    fn push_transform(&mut self, t: TransformRule) {
        self.transforms.push(t);
    }

    pub fn commands(&self) -> &[CommandRule] {
        &self.commands
    }

    pub fn transforms(&self) -> &[TransformRule] {
        &self.transforms
    }

    pub fn command_matches(&self, command: &CommandRule) -> bool {
        self.commands
            .iter()
            .find(|cr| cr.space() == command.space() && cr.op() == command.op())
            .is_some()
    }

    // deduplicates command rules
    pub fn dedup_commands(&mut self) {
        let mut iter = vec![];
        std::mem::swap(&mut self.commands, &mut iter);
        let mut iter = iter.into_iter();

        while let Some(c) = iter.next() {
            if !self.command_matches(&c) {
                self.commands.push(c);
            }
        }
    }

    pub fn matches_commands(&self, al: &Aliased) -> bool {
        self.commands
            .iter()
            .find(|c| {
                if let (AliasToken::Space(st), Some(spc)) = (al.token(), c.space()) {
                    st == spc
                } else if let (AliasToken::Operation(o), Some(op)) = (al.token(), c.op()) {
                    o == op
                } else if let (AliasToken::Flag(flg), Some(flags)) = (al.token(), c.flags()) {
                    flags.iter().any(|f| f.ident() == flg)
                } else {
                    false
                }
            })
            .is_some()
    }
}

impl Parse for Rules {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        let mut rb = Self::default();
        while stream.peek(Ident::peek_any) {
            match &Ident::parse(stream)?.to_string()[..] {
                "t" => unimplemented!(),
                "c" => rb.push_commands(ExpandedCommandRule::parse(stream)?),
                val => panic!("expected c or t found {:?}", val),
            }
        }

        Ok(rb)
    }
}

#[derive(Debug)]
pub struct Attributes {
    fish_cmp: bool,
    nu_cmp: bool,
    gen_help: bool,
    gen_ver: bool,
    auto_alias: bool,
    // name of the top level type of the cli tool
    // also used in other types of the type tree when necessary
    root_name: String,
    ignore_naming_conventions: bool,
    branch_off_root: bool,
    derives: Vec<Ident>,
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            fish_cmp: true,
            nu_cmp: true,
            gen_help: true,
            gen_ver: true,
            auto_alias: true,
            ignore_naming_conventions: false,
            branch_off_root: false,
            root_name: "".into(),
            derives: Self::default_derives(),
        }
    }
}

impl Attributes {
    fn new(name: String) -> Self {
        Self {
            root_name: name,
            ..Self::default()
        }
    }

    fn default_derives() -> Vec<Ident> {
        vec![
            Ident::new("Debug", Span::call_site()),
            Ident::new("Clone", Span::call_site()),
            Ident::new("PartialEq", Span::call_site()),
            Ident::new("Eq", Span::call_site()),
            // TODO add support for default values for parameterized flags
            // so that Default can be implemented
            // Ident::new("Default", Span::call_site())
        ]
    }

    pub fn auto_alias(&self) -> bool {
        self.auto_alias
    }
}

impl Parse for Attributes {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        let name = ResolveCrate::new().read_manifest().crate_name();
        let mut attrs = Attributes::new(name);

        _ = <Token![#]>::parse(stream)?;
        let content;
        let bracket = bracketed!(content in stream);
        // WARN doesnt work
        // cus not all attrs are boolean
        while content.peek(Ident::peek_any) {
            match &Ident::parse(&content)?.to_string()[..] {
                // "fish_cmp"
                // | "nu_cmp"
                // | "ignore_naming_conventions"
                // | "no_help"
                // | "no_version" => unimplemented!(),
                "root_name" => {
                    _ = <Token![=]>::parse(&content)?;
                    let name = Lit::parse(&content)?;
                    let Lit::Str(ls) = name else {
                        unreachable!("root_name attr has to take a single str lit")
                    };

                    attrs.root_name = ls.value();
                }
                "fish_cmp" => attrs.fish_cmp = true,
                "nu_cmp" => attrs.nu_cmp = true,
                "no_help" => attrs.gen_help = false,
                "no_version" => attrs.gen_ver = false,
                "branch_off_root" => attrs.branch_off_root = true,
                "ignore_naming_conventions" => attrs.ignore_naming_conventions = true,
                "no_auto_alias" => attrs.auto_alias = false,
                "disable_derives" => {
                    let derives;
                    let paren = parenthesized!(derives in content);
                    let disable = derives
                        .parse_terminated(Ident::parse, Token![,])?
                        .into_iter()
                        .for_each(|d| {
                            if let Some(pos) = attrs.derives.iter().position(|i| *i == d) {
                                attrs.derives.remove(pos);
                            }
                        });
                }
                val => panic!("unrecognized fake attribute {}", val),
            }
            if !content.is_empty() {
                _ = <Token![,]>::parse(&content)?;
            }
        }

        Ok(attrs)
    }
}

#[derive(Debug, Default)]
pub struct Aliases {
    aliases: Vec<Aliased>,
}

#[derive(Debug, PartialEq)]
pub struct Aliased {
    token: AliasToken,
    alias: Ident,
}

impl Aliased {
    pub fn from_values(token: AliasToken, alias: Ident) -> Self {
        Self { alias, token }
    }
}

#[derive(Debug, PartialEq)]
pub enum AliasToken {
    Space(Ident),
    Operation(Ident),
    Flag(Ident),
}

impl AliasToken {
    pub fn alias(&self) -> Option<Ident> {
        let s = self.ident().to_string();
        if s.len() < 4 {
            return None;
        }

        Some(match self {
            Self::Space(i) => Ident::new(&s[..4], Span::call_site()),
            Self::Operation(i) | Self::Flag(i) => Ident::new(&s[..1], Span::call_site()),
        })
    }

    pub fn is_space(&self) -> bool {
        discriminant(self) == discriminant(&Self::Space(dummy_ident()))
    }

    pub fn is_operation(&self) -> bool {
        discriminant(self) == discriminant(&Self::Operation(dummy_ident()))
    }

    pub fn is_flag(&self) -> bool {
        discriminant(self) == discriminant(&Self::Flag(dummy_ident()))
    }

    pub fn space(&self) -> Option<&Ident> {
        let Self::Space(i) = self else { return None };

        Some(i)
    }

    pub fn operation(&self) -> Option<&Ident> {
        let Self::Operation(i) = self else {
            return None;
        };

        Some(i)
    }

    pub fn flag(&self) -> Option<&Ident> {
        let Self::Flag(i) = self else { return None };
        Some(i)
    }

    pub fn ident(&self) -> &Ident {
        match self {
            Self::Flag(i) | Self::Operation(i) | Self::Space(i) => i,
        }
    }
}

impl TryFrom<[Ident; 2]> for AliasToken {
    type Error = syn::Error;

    fn try_from(mut idents: [Ident; 2]) -> ParseResult<Self> {
        let [a, b] = std::mem::replace(&mut idents, [dummy_ident(), dummy_ident()]);

        match a {
            i if i == Ident::new("s", Span::call_site()) => Ok(Self::Space(b)),
            i if i == Ident::new("o", Span::call_site()) => Ok(Self::Operation(b)),
            i if i == Ident::new("f", Span::call_site()) => Ok(Self::Flag(b)),
            _ => Err(syn::Error::new(
                Span::call_site(),
                "bad scope for alias; use s, o or f",
            )),
        }
    }
}

impl TryFrom<[Ident; 3]> for Aliased {
    type Error = syn::Error;

    fn try_from(mut idents: [Ident; 3]) -> ParseResult<Self> {
        let [a, b, c] =
            std::mem::replace(&mut idents, [dummy_ident(), dummy_ident(), dummy_ident()]);

        Ok(Self {
            alias: c,
            token: [a, b].try_into()?,
        })
    }
}

fn extract_aliased(content: ParseStream) -> ParseResult<Aliased> {
    let temp;
    let scope = Ident::parse(&content)?;
    _ = parenthesized!(temp in content);
    let origin = Ident::parse(&temp)?;
    _ = <Token![=]>::parse(&content)?;
    let alias = Ident::parse(&content)?;

    [scope, origin, alias].try_into()
}

impl Parse for Aliases {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        let mut aliases = vec![];
        // parse the aliases ident
        _ = Ident::parse(stream)?;
        let content;
        let brace = braced!(content in stream);

        while content.peek(Ident::peek_any) {
            aliases.push(extract_aliased(&content)?);

            if !content.is_empty() {
                _ = <Token![,]>::parse(&content)?;
            }
        }

        Ok(Aliases { aliases })
    }
}

impl Aliases {
    pub fn aliases(&self) -> &[Aliased] {
        &self.aliases
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Aliased> {
        self.aliases.iter_mut()
    }

    pub fn from_values(values: Vec<Aliased>) -> Self {
        Self { aliases: values }
    }

    pub fn into_iter(self) -> impl Iterator<Item = Aliased> {
        self.aliases.into_iter()
    }

    pub fn push(&mut self, aliased: Aliased) {
        self.aliases.push(aliased);
    }

    pub fn contains(&self, a: &Aliased) -> bool {
        self.aliases
            .iter()
            .find(|al| {
                std::mem::discriminant(al.token()) == std::mem::discriminant(a.token())
                    && a.alias() == al.alias()
            })
            .is_some()
    }
}

impl Aliased {
    pub fn token(&self) -> &AliasToken {
        &self.token
    }

    pub fn alias(&self) -> &Ident {
        &self.alias
    }
}

macro_rules! alias_token {
    ('s', $i: ident) => {
        AliasToken::Space($i.clone())
    };
    ('o', $i: ident) => {
        AliasToken::Operation($i.clone())
    };
    ('f', $i: ident) => {
        AliasToken::Flag($i.clone())
    };
    ($s: ident, $i: ident) => {
        match $s {
            's' => AliasToken::Space($i.clone()),
            'o' => AliasToken::Operation($i.clone()),
            'f' => AliasToken::Flag($i.clone()),
            v => panic!("alias_token: received bad input for scope {}", v),
        }
    };
}

pub(crate) use alias_token;

#[derive(Debug)]
pub struct CommandStack {
    attrs: Attributes,
    rules: Rules,
    aliases: Aliases,
}

impl Parse for CommandStack {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        let (mut attrs, mut rules, mut aliases) =
            (Default::default(), Default::default(), Default::default());
        while !stream.is_empty() {
            if stream.peek(Token![#]) {
                attrs = Attributes::parse(stream)?;
            } else if stream.peek(Ident::peek_any) {
                if stream.fork().parse::<Ident>()?.to_string().as_str() == "aliases" {
                    aliases = Aliases::parse(stream)?;
                } else {
                    rules = Rules::parse(stream)?;
                }
            }
        }

        Ok(Self {
            attrs,
            aliases,
            rules,
        })
    }
}

impl CommandStack {
    pub fn rules_mut(&mut self) -> &mut Rules {
        &mut self.rules
    }

    pub fn rules_ref(&self) -> &Rules {
        &self.rules
    }

    pub fn attrs(&self) -> &Attributes {
        &self.attrs
    }

    pub fn aliases_ref(&self) -> &Aliases {
        &self.aliases
    }

    pub fn aliases_mut(&mut self) -> &mut Aliases {
        &mut self.aliases
    }

    pub fn set_aliases(&mut self, als: &mut Aliases) {
        self.aliases = std::mem::take(als);
    }
}
