use super::lex::Token;

pub trait Parse {
    fn parse<T: std::iter::Iterator<Item = Token>>(tokens: T);

    fn has_cmd() -> bool;

    fn has_scope() -> bool;

    fn cmd_syntax() -> Syntax;

    fn scope_syntax() -> Syntax;

    fn flag_syntax() -> Syntax;

    fn take_args() -> Vec<Arg>;
}

/// possible arg types
#[derive(Debug)]
pub enum Arg {
    Int,
    Float,
    Str,
    Bool,
    Date,
    Phone,
    Email,
    URL,
}

#[derive(Debug)]
pub enum Syntax {
    // -a
    SingleHyphenLowerCase,
    // -A
    SingleHyphenUpperCase,
    // --yada-yada
    DoubleHyphen,
    // this-and-that
    HyphenLess,
}

pub struct DefaultParser;
