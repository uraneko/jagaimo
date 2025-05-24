//! the engine takes the macro inputs and parses them
//! into a data form (RuleBook) that can be used by the macro
//! so to say, the macro's direct input is the RuleBook struct
//!
//! RULES TYPES:
//! * r(egions) => defines all the regions of the root scope, if any,
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
//! <- collections region can be aliased as colls, parser accepts both
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
//! i.e., if you have a cli tool called `ct` with 4 regions `a` `b` `c` and `d`
//! thi attribute will generate code for 4 different cli tools `ct-a`, `ct-b`, `ct-c` and `ct-d`
//! instead of generating code for a single cli tool;`ct`
//! * root_name
//! <- renames the resulting top level cli type, default is crate name
//! follows the rust naming convetions
//! * ignore_naming_conventions <- turns off rust naming convetions for cli top level type

use proc_macro2::Span;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream, Result as ParseResult};
use syn::{Ident, Lit, Token, Type};
use syn::{braced, bracketed, parenthesized};

use crate::read_manifest::read_manifest;

#[derive(Debug, Default)]
struct RegionsRule {
    regions: Vec<String>,
}

// rgns { [ bababa ] }
impl Parse for RegionsRule {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        let mut content;
        let brace = braced!(content in stream);
        let bracket = bracketed!(content in content);
        let arr = content
            .parse_terminated(Ident::parse, Token![,])?
            .into_iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>();

        Ok(Self { regions: arr })
    }
}

#[derive(Debug)]
enum Scope {
    Root,
    Region(String),
    RegionedOperation { region: String, op: String },
    Operation(String),
}

#[derive(Debug)]
struct OperationRule {
    scope: Scope,
    ops: Vec<String>,
}

impl Parse for OperationRule {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        let mut content;
        let brace = braced!(content in stream);
        let scope = Ident::parse(&content);
        let scope = if scope.is_err() {
            Scope::Root
        } else {
            Scope::Region(scope?.to_string())
        };

        let bracket = bracketed!(content in content);
        let arr = content
            .parse_terminated(Ident::parse, Token![,])?
            .into_iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>();

        Ok(Self { scope, ops: arr })
    }
}

#[derive(Debug)]
struct FlagRule {
    scope: Scope,
    flags: Vec<String>,
}

impl From<Vec<Ident>> for Scope {
    fn from(mut value: Vec<Ident>) -> Self {
        match value.len() {
            0 => Self::Root,
            2 => {
                let item = value.pop().unwrap();
                if value[0] == Ident::new("r", Span::call_site()) {
                    Self::Region(item.to_string())
                } else {
                    Self::Operation(item.to_string())
                }
            }
            4 => Self::RegionedOperation {
                region: value.remove(1).to_string(),
                op: value.pop().unwrap().to_string(),
            },
            _ => panic!("scope cant take only: 0, 2 or 4 idents"),
        }
    }
}

impl Parse for FlagRule {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        let mut content;
        let brace = braced!(content in stream);

        let mut idents: Vec<Ident> = vec![];
        let mut scope = vec![];
        let mut temp;
        loop {
            if content.peek(Ident::peek_any) {
                scope.extend([Ident::parse(&content)?, {
                    let _paren = parenthesized!(temp in content);

                    Ident::parse(&temp)?
                }]);
            } else {
                break;
            }
        }
        let bracket = bracketed!(content in content);
        let arr = content
            .parse_terminated(Ident::parse, Token![,])?
            .into_iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>();

        Ok(Self {
            scope: scope.into(),
            flags: arr,
        })
    }
}

#[derive(Debug)]
struct ParameterRule {
    scope: Scope,
    params: Vec<Type>,
}

impl Parse for ParameterRule {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        let mut content;
        let brace = braced!(content in stream);

        let mut idents: Vec<Ident> = vec![];
        let mut scope = vec![];
        let mut temp;
        while content.peek(Ident::peek_any) {
            scope.extend([Ident::parse(&content)?, {
                let _paren = parenthesized!(temp in content);

                Ident::parse(&temp)?
            }]);
        }
        let bracket = bracketed!(content in content);
        let arr = content
            .parse_terminated(Type::parse, Token![,])?
            .into_iter()
            .map(|v| v)
            .collect::<Vec<Type>>();

        Ok(Self {
            scope: scope.into(),
            params: arr,
        })
    }
}

// NOTE this needs the Token type to be written first
// which means this has to waut for the parser module
// since Token (nikujaga's parser's token, not syn's) is part of parser::lex
#[derive(Debug)]
struct TransformRule {
    src: Pattern,
    dest: Pattern,
}

impl Parse for TransformRule {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        todo!()
    }
}

#[derive(Debug)]
struct Pattern {
    ident: Ident,
    ty: Type,
}

#[derive(Debug)]
enum Flags {
    AliasWhenPossible,
    NoAlias,
    UpperShort,
}

#[derive(Debug, Default)]
pub(crate) struct RuleBook {
    regions: RegionsRule,
    opes: Vec<OperationRule>,
    flags: Vec<FlagRule>,
    params: Vec<ParameterRule>,
    transforms: Vec<TransformRule>,
}

impl RuleBook {
    fn regions(&mut self, r: RegionsRule) {
        self.regions = r;
    }

    fn operation(&mut self, ope: OperationRule) {
        self.opes.push(ope);
    }

    fn flag(&mut self, f: FlagRule) {
        self.flags.push(f);
    }

    fn param(&mut self, p: ParameterRule) {
        self.params.push(p);
    }

    fn transform(&mut self, t: TransformRule) {
        self.transforms.push(t);
    }
}

impl Parse for RuleBook {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        let mut rb = Self::default();
        while stream.peek(Ident::peek_any) {
            match &Ident::parse(stream)?.to_string()[..] {
                "r" => rb.regions(RegionsRule::parse(stream)?),
                "o" => rb.operation(OperationRule::parse(stream)?),
                "f" => rb.flag(FlagRule::parse(stream)?),
                "p" => rb.param(ParameterRule::parse(stream)?),
                val => panic!("expected one of r, o, f, p, tr or tf; found {:?}", val),
            }
        }

        Ok(rb)
    }
}

#[derive(Debug)]
struct Attributes {
    fish_cmp: bool,
    nu_cmp: bool,
    gen_help: bool,
    gen_ver: bool,
    root_name: String,
    ignore_naming_conventions: bool,
    branch_off_root: bool,
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
}

impl Parse for Attributes {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        let [name, _] = read_manifest();
        let mut attrs = Attributes::new(name);

        _ = <Token![#]>::parse(stream)?;
        let content;
        let bracket = bracketed!(content in stream);
        // WARN doesnt work
        // cus not all attrs are boolean
        while content.peek(Ident::peek_any) {
            match &Ident::parse(&content)?.to_string()[..] {
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
                "ignore_naming_conventions" => attrs.ignore_naming_conventions = true,
                _ => panic!("unrecognized fake attribute"),
            }
            if !content.is_empty() {
                _ = <Token![,]>::parse(&content)?;
            }
        }

        Ok(attrs)
    }
}

#[derive(Debug)]
enum CommandItem {
    Region,
    Operation,
    Flag,
    Parameter,
}

use crate::parser::LexToken;

#[derive(Debug)]
struct Item {
    ty: std::mem::Discriminant<LexToken>,
    name: String,
    alias: Option<String>,
}

#[derive(Debug, Default)]
enum CompoundingItem {
    #[default]
    Kebab,
    Snake,
    Both,
}

#[derive(Debug, Default)]
enum CharsCase {
    #[default]
    LowerLong,
    UpperShort,
}

#[derive(Debug, Default)]
struct Syntax {
    case: CharsCase,
    bridge: CompoundingItem,
    alias_eagerly: bool,
    aliases: Vec<Item>,
}

impl Parse for Syntax {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        let mut syntax = Syntax::default();
        let content;
        let brace = braced!(content in stream);

        while content.peek(Ident::peek_any) {
            match Ident::parse(&content)?.to_string() {
                "AliasEagerly" => 
                "" => 
                "" => 
                "" => 
            }
        }
    }
}
