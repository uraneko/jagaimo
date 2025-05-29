use syn::parse::{Parse, ParseStream, Result as ParseResult};
use syn::{Ident, Type};

pub use super::flags::Flag;

#[derive(Debug, PartialEq)]
pub enum Context {
    Param(Type),
    Flags(Vec<Flag>),
    ParamAndFlags { param: Type, flags: Vec<Flag> },
}
