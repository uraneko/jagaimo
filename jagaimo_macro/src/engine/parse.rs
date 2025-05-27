//! the engine takes the macro inputs and parses them
//! into a data form (RuleBook) that can be used by the macro
//! so to say, the macro's direct input is the RuleBook struct
//!
//! RULES TYPES:
//! * r(egions) => defines all the realms of the root scope, if any,
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
//! <- collections realm can be aliased as colls, parser accepts both
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
//! i.e., if you have a cli tool called `ct` with 4 realms `a` `b` `c` and `d`
//! thi attribute will generate code for 4 different cli tools `ct-a`, `ct-b`, `ct-c` and `ct-d`
//! instead of generating code for a single cli tool;`ct`
//! * root_name
//! <- renames the resulting top level cli type, default is crate name
//! follows the rust naming convetions
//! * ignore_naming_conventions <- turns off rust naming convetions for cli top level type

use super::IdentExt;
use super::Span;
use super::{Ident, Lit, Token, Type};
use super::{Parse, ParseResult, ParseStream};
use super::{braced, bracketed, parenthesized};
use super::{discriminant, dummy_ident};

use crate::resolve_crate::ResolveCrate;
use crate::traits::Token;

pub mod flags;
pub mod operations;
pub mod parameters;
pub mod realms;
pub mod transforms;

pub use flags::FlagsRule;
pub use operations::OperationsRule;
pub use parameters::ParamsRule;
pub use realms::RealmsRule;
pub use transforms::TransformsRule;

#[derive(Debug, Default, PartialEq)]
pub enum Scope {
    #[default]
    Executable,
    Realm(Ident),
    RealmOperation {
        realm: Ident,
        op: Ident,
    },
    Operation(Ident),
}

impl Scope {
    // returns option of the ident of realm only if self is of the realm variant
    // otherwise returns None
    pub fn realm(&self) -> Option<&Ident> {
        let Scope::Realm(i) = self else {
            return None;
        };

        Some(i)
    }

    pub fn realm_op(&self) -> Option<[&Ident; 2]> {
        let Scope::RealmOperation { realm, op } = self else {
            return None;
        };

        Some([realm, op])
    }

    pub fn op(&self) -> Option<&Ident> {
        let Scope::Operation(i) = self else {
            return None;
        };

        Some(i)
    }

    pub fn is_escutable(&self) -> bool {
        self == &Scope::Executable
    }

    pub fn is_realm(&self) -> bool {
        discriminant(self) == discriminant(&Self::Realm(dummy_ident()))
    }

    pub fn is_realm_op(&self) -> bool {
        discriminant(self)
            == discriminant(&Self::RealmOperation {
                realm: dummy_ident(),
                op: dummy_ident(),
            })
    }

    pub fn is_op(&self) -> bool {
        discriminant(self) == discriminant(&Self::Operation(dummy_ident()))
    }

    pub fn matches_realm(&self, r: &Ident) -> bool {
        if let Some(realm) = self.realm() {
            return realm == r;
        }

        false
    }

    pub fn matches_realm_op(&self, ro: &[&Ident; 2]) -> bool {
        if let Some([r, o]) = self.realm_op() {
            return r == ro[0] && o == ro[1];
        }

        false
    }

    pub fn matches_op(&self, o: &Ident) -> bool {
        if let Some(op) = self.op() {
            return op == o;
        }

        false
    }
}

impl From<Vec<Ident>> for Scope {
    fn from(mut value: Vec<Ident>) -> Self {
        match value.len() {
            0 => Self::Executable,
            2 => {
                let item = value.pop().unwrap();
                if value[0] == Ident::new("r", Span::call_site()) {
                    Self::Realm(item)
                } else {
                    Self::Operation(item)
                }
            }
            4 => Self::RealmOperation {
                realm: value.remove(1),
                op: value.pop().unwrap(),
            },
            _ => panic!("scope cant take only: 0, 2 or 4 idents"),
        }
    }
}

#[derive(Debug, Default)]
pub struct RuleBook {
    realms: RealmsRule,
    opes: Vec<OperationsRule>,
    flags: Vec<FlagsRule>,
    params: Vec<ParamsRule>,
    transforms: Vec<TransformsRule>,
}

impl RuleBook {
    pub fn realms(&self) -> &RealmsRule {
        &self.realms
    }

    pub fn realms_vec(&self) -> &[Ident] {
        &self.realms.realms()
    }

    pub fn ops(&self) -> &[OperationsRule] {
        &self.opes
    }

    pub fn flags(&self) -> &[FlagsRule] {
        &self.flags
    }

    pub fn params(&self) -> &[ParamsRule] {
        &self.params
    }
}

impl RuleBook {
    fn set_realms(&mut self, r: RealmsRule) {
        self.realms = r;
    }

    fn push_operation(&mut self, ope: OperationsRule) {
        self.opes.push(ope);
    }

    fn push_flag(&mut self, f: FlagsRule) {
        self.flags.push(f);
    }

    fn push_param(&mut self, p: ParamsRule) {
        self.params.push(p);
    }

    fn push_transform(&mut self, t: TransformsRule) {
        self.transforms.push(t);
    }
}

impl Parse for RuleBook {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        let mut rb = Self::default();
        while stream.peek(Ident::peek_any) {
            match &Ident::parse(stream)?.to_string()[..] {
                "r" => rb.set_realms(RealmsRule::parse(stream)?),
                "o" => rb.push_operation(OperationsRule::parse(stream)?),
                "f" => rb.push_flag(FlagsRule::parse(stream)?),
                "p" => rb.push_param(ParamsRule::parse(stream)?),
                "tr" | "tf" => unimplemented!(),
                val => panic!("expected one of r, o, f, p, tr or tf; found {:?}", val),
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

#[derive(Debug, Default, PartialEq)]
pub enum Case {
    Snake,
    #[default]
    Kebab,
    Both,
}

#[derive(Debug, Default)]
pub struct Syntax {
    case: Case,
    alias_eagerly: bool,
    aliases: Vec<Alias>,
}

#[derive(Debug)]
pub struct Alias {
    token: Token,
    alias: String,
}

impl Parse for Syntax {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        let mut syntax = Syntax::default();
        // parse the syntax ident
        _ = Ident::parse(stream)?;
        let content;
        let brace = braced!(content in stream);

        let mut temp;
        let mut subtemp;
        while content.peek(Ident::peek_any) {
            match Ident::parse(&content)?.to_string().as_str() {
                "AliasEagerly" => syntax.alias_eagerly = true,
                "SnakeOnly" => {
                    if syntax.case != Case::Both {
                        syntax.case = Case::Snake
                    }
                }
                "AllowSnakeCase" => syntax.case = Case::Both,
                "Alias" => {
                    _ = parenthesized!(temp in content);
                    let scope = Ident::parse(&temp)?;
                    _ = parenthesized!(subtemp in temp);
                    let origin = Ident::parse(&subtemp)?;
                    _ = <Token![=]>::parse(&temp)?;
                    let alias = Ident::parse(&temp)?;

                    syntax.aliases.push([scope, origin, alias].into());
                }
                val => unimplemented!("unemplemented syntax rule {}", val),
            }

            if !content.is_empty() {
                _ = <Token![,]>::parse(&content)?;
            }
        }

        Ok(syntax)
    }
}

impl From<[Ident; 2]> for Token {
    fn from(value: [Ident; 2]) -> Self {
        let val = value[1].to_string();
        match value[0].to_string().as_str() {
            "r" => Token::Realm(val),
            "o" => Token::Operation(val),
            // WARN this cant generate all flag tokens properly
            // since parameterized flag tokens contain their params data and
            // this doesnt have access to those params, which are runtime values
            // NOTE then again, the alias itself has no use for the params
            // it functions correctly without that info
            // the only problem is that Token::Flag is a boolean flag
            // which is a misrepresentation for parameterized flags
            "f" => Token::Flag(val),
            _ => panic!("aliases can only be made for realm (r), operation (o) or a flag (f)"),
        }
    }
}

impl From<[Ident; 3]> for Alias {
    fn from(mut value: [Ident; 3]) -> Self {
        use std::mem::swap;

        let [mut a, mut b, mut c] = [dummy_ident(), dummy_ident(), dummy_ident()];

        swap(&mut a, &mut value[0]);
        swap(&mut b, &mut value[1]);
        swap(&mut c, &mut value[2]);

        Self {
            token: [a, b].into(),
            alias: c.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct CommandTree {
    attrs: Attributes,
    rules: RuleBook,
    syntax: Syntax,
}

impl Parse for CommandTree {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        let (mut attrs, mut rules, mut syntax) =
            (Default::default(), Default::default(), Default::default());
        while !stream.is_empty() {
            if stream.peek(Token![#]) {
                attrs = Attributes::parse(stream)?;
            } else if stream.peek(Ident::peek_any) {
                if stream.fork().parse::<Ident>()?.to_string().as_str() == "syntax" {
                    syntax = Syntax::parse(stream)?;
                } else {
                    rules = RuleBook::parse(stream)?;
                }
            }
        }

        Ok(Self {
            attrs,
            syntax,
            rules,
        })
    }
}

impl CommandTree {
    pub fn rules(&self) -> &RuleBook {
        &self.rules
    }

    pub fn attrs(&self) -> &Attributes {
        &self.attrs
    }

    pub fn syntax(&self) -> &Syntax {
        &self.syntax
    }
}
