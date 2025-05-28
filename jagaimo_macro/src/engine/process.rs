use std::collections::HashMap;

pub use proc_macro::TokenStream as TS;
pub use proc_macro2::TokenStream as TS2;
use quote::quote;
use syn::parse_str;
use syn::{Ident, Type, Variant};

use super::parse::Case;

mod commands;
mod flags;
mod transforms;

use crate::engine::parse::flags::Flag;

pub fn ident_to_variant(i: &Ident) -> Variant {
    parse_str::<Variant>(&format!("{}({})", i, i)).unwrap()
}

// this takes the data in rawcommandtree and transforms it into a format that is useful
// for generating cli types, lex, parse, help ...
pub(crate) struct CookedCommandTree {
    // name of the top level data type that will have Command implemented for it
    top_level_name: Ident,
    // specifies wether to fail items on snake, kebab or not fail at all on both cases
    case: Case,
    // types to be generated
    type_tree: Vec<Branch>,
    // includes the manually provided aliases + auto aliases if their flag is on
    // also resolves alias conflicts, if any
    aliases: Aliases,
    // all possible commands token streams
    // graph: HashMap<Caller, Params>,
    // if this is on then, this struct would generate
    // multiple top level data types branching off all the top command callers
    // instead of 1 for the whole cli
    // unimplemented
    // branch_off: bool,
    version: Option<String>,
    // help: Option<Help>,
}

#[derive(Debug)]
struct Branch {
    seq: Vec<CommandToken>,
}

#[derive(Debug)]
enum CommandToken {
    Space(Ident),
    Operation(Ident),
    Flag(Flag),
    Param(Type),
}

#[derive(Debug)]
struct Aliases {
    map: HashMap<CommandToken, Vec<Ident>>,
}

#[derive(Debug)]
struct Transforms {}
