use std::env::{Args, args};

#[derive(Debug)]
pub enum Token {
    // fs path of the command root; the executable being run
    ExecutablePath(String),
    // realm name
    Realm(String),
    // operation name
    Operation(String),
    // flag name; bool flag, takes no paarams
    Flag(String),
    // flag name + param; takes one param
    FlagWithParam { flag: String, param: String },
    // flag name + params; takes many params, possibly of different types
    FlagWithParams { falg: String, params: Vec<String> },
}

// DOCS
// 1/ take the input, which is the args
// 2/ resolve aliases
// <- may not resolve aliases correctly without scope information, if different items have
// the same name
// 3/ resolve transforms
// <- requires aliases to be resolved already
// 4/ validate caller
// 4/ validate flags
// 5/ validate params
// 6/ map strings to tokens
// <- should come first, but cant since it needs validations
// 7/ return tokens
//
// 1/ check if 1st is realm or operation

// the follwing 2 types should be under the engine::process module
use std::collections::HashMap;
use syn::Type;

// DOCS lex and parse shouldnt be traits
// they should be single fns
// engine parse should generate a graph of every possible
// command
// then generate the token extraction logic for every command
// finally match on the input args and every command's token extraction logic

pub struct Caller {
    realm: Option<String>,
    operation: Option<String>,
}

pub struct Params {
    flags: HashMap<String, Type>,
}

pub trait Lex<I, const RS: usize, const OS: usize, const FS: usize, const GS: usize>
where
    Self: IntoIterator<Item = Token>,
    Self: Into<Vec<Token>>,
    Self: AsRef<[Token]>,
    Self: From<Vec<Token>>,
{
    const REGIONS: [&str; RS];
    const OPERATIONS: [&str; OS];
    const FLAGS: [&str; FS];

    const GRAPH: [Vec<Token>; GS];
    // HashMap<Caller, Params>;

    fn has_realms() -> bool {
        !Self::REGIONS.is_empty()
    }

    fn has_operations() -> bool {
        !Self::OPERATIONS.is_empty()
    }

    fn has_flags() -> bool {
        !Self::FLAGS.is_empty()
    }

    fn is_realm(s: &str) -> bool {
        Self::REGIONS.contains(&s)
    }

    fn is_operation(s: &str) -> bool {
        Self::OPERATIONS.contains(&s)
    }

    fn is_flag(s: &str) -> bool {
        Self::FLAGS.contains(&s)
    }

    fn input() -> impl IntoIterator {
        args()
    }

    fn parse_realm(v: &mut Vec<Token>, next: String) -> Option<String> {
        if !Self::has_realms() {
            return None;
        }

        if Self::is_realm(&next) {
            v.push(Token::Realm(next));
            return None;
        }

        Some(next)
    }

    fn parse_operation(v: &mut Vec<Token>, next: String) -> Option<String> {
        if !Self::has_operations() {
            return None;
        }

        if Self::is_operation(&next) {
            v.push(Token::Operation(next));
            return None;
        }

        Some(next)
    }

    fn parse_flag(v: &mut Vec<Token>, next: String) -> Option<String> {
        None
    }

    // this is called if no token matches given rules
    // the lexer then checks if any alias matches the token
    // if so then lexer resolves alias and saves the real value to a token
    fn aliases(t: Token) -> Option<Token> {
        None
    }

    // this is called after calling aliases to resolve the transforms
    fn transforms(t: Token) -> Option<Token> {
        None
    }

    // validates the command against the known callers
    // root realm ... <- is a caller
    // root ... <- is a caller
    // root realm operation ... <- is a caller too
    // caller is root realm and operation,, whichever of them exists is part of caller
    // if the given caller is not found in the command graph then we error out
    fn validate_callers();

    // upon validating callers, we move to validating the flag names to the callers
    // if a command with such combination doesn't exist in the command graph then we error out
    fn validate_flags();

    // finally we validate the parameter types found in the command graph with those in the given
    // command
    // if the types dont match anything in the graph,
    // again, the command is invalid and we error out
    //
    // once this step is passed safely, the command is clear
    // we then map the string values in the command into tokens
    fn validate_params();

    // maps the string values in the input to their proper token values
    //
    // this finalizes the lexing process
    fn map_tokens();

    // lexes the input and returns a vec of tokens
    fn lex() -> Vec<Token>;

    fn command_graph() -> Vec<Token>;
}
