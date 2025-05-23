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
//! * AliasWhenPossble(scope(s), ...) => auto generate aliases for the given scope items when
//! possible
//! * Alias(scope(item) = alias) => manual alias, used in the form:
//! Alias(r(collections) = colls)
//! <- collections region can be aliased as colls  
//! <- superseeds auto alias generation
//! * KebabOnly(scope(s), ...) => only kebab (-) case is valid for the given scope(s) items
//! * SnakeOnly(scope(s), ...) => only snake (_) case is valid for the given scope(s) items
//!
//! ATTRIBUTES:
//! * no_help
//! * no_version
//! * fish_cmp
//! * nu_cmp
use proc_macro2::Span;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream, Result as ParseResult};
use syn::{Expr, Ident, Token, Type};
use syn::{braced, bracketed, parenthesized};

#[derive(Debug, Default)]
struct RegionsRule {
    regions: Vec<Ident>,
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
            .map(|v| v)
            .collect::<Vec<Ident>>();

        Ok(Self { regions: arr })
    }
}

#[derive(Debug)]
enum Scope {
    Root,
    Region(Ident),
    RegionedOperation { region: Ident, op: Ident },
    Operation(Ident),
}

#[derive(Debug)]
struct OperationRule {
    scope: Scope,
    ops: Vec<Ident>,
}

impl Parse for OperationRule {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        let mut content;
        let brace = braced!(content in stream);
        let scope = Ident::parse(&content);
        let scope = if scope.is_err() {
            Scope::Root
        } else {
            Scope::Region(scope?)
        };
        println!("{:?}", scope);
        let bracket = bracketed!(content in content);
        let arr = content
            .parse_terminated(Ident::parse, Token![,])?
            .into_iter()
            .map(|v| v)
            .collect::<Vec<Ident>>();

        Ok(Self { scope, ops: arr })
    }
}

#[derive(Debug)]
struct FlagRule {
    scope: Scope,
    flags: Vec<Ident>,
}

impl From<Vec<Ident>> for Scope {
    fn from(mut value: Vec<Ident>) -> Self {
        match value.len() {
            0 => Self::Root,
            2 => {
                let item = value.pop().unwrap();
                if value[0] == Ident::new("r", Span::call_site()) {
                    Self::Region(item)
                } else {
                    Self::Operation(item)
                }
            }
            4 => Self::RegionedOperation {
                region: value.remove(1),
                op: value.pop().unwrap(),
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
            .map(|v| v)
            .collect::<Vec<Ident>>();

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

#[derive(Debug)]
struct Attributes {
    fish_cmp: bool,
    nu_cmp: bool,
    gen_help: bool,
    gen_ver: bool,
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            fish_cmp: true,
            nu_cmp: true,
            gen_help: true,
            gen_ver: true,
        }
    }
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

// use std::sync::LazyLock;
// const R: LazyLock<Ident> = LazyLock::new(|| Ident::new("r", Span::call_site()));
// const O: LazyLock<Ident> = LazyLock::new(|| Ident::new("o", Span::call_site()));
// const F: LazyLock<Ident> = LazyLock::new(|| Ident::new("f", Span::call_site()));
// const P: LazyLock<Ident> = LazyLock::new(|| Ident::new("p", Span::call_site()));
// const TR: LazyLock<Ident> = LazyLock::new(|| Ident::new("t.r", Span::call_site()));
// const TF: LazyLock<Ident> = LazyLock::new(|| Ident::new("t.f", Span::call_site()));

// TODO add support for random rules positioning
impl Parse for RuleBook {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        let mut rb = Self::default();
        while stream.peek(Ident::peek_any) {
            match &Ident::parse(stream)?.to_string()[..] {
                "r" => rb.regions(RegionsRule::parse(stream)?),
                "o" => rb.operation(OperationRule::parse(stream)?),
                "f" => rb.flag(FlagRule::parse(stream)?),
                "p" => rb.param(ParameterRule::parse(stream)?),
                val => panic!("expected R, O, F, P, TR or TF; found {:?}", val),
            }
        }

        Ok(rb)
    }
}
