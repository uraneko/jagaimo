use proc_macro::TokenStream as TS;
use proc_macro2::Span;
use proc_macro2::TokenStream as TS2;
use quote::ToTokens;
use quote::quote;
use syn::Ident;
use syn::parse_macro_input;

// DOCS
//                  command
//  _______________________________
// |                               |
// root space operation flags params
// |                  | |          |
// ---------------------------------
//        scope           context

mod generate;
mod parse;
mod process;
mod resolve_crate;

use parse::{CommandStack, Rules};
use process::TypeTreeStack;
use process::parse_help::extract_from_tables;

#[proc_macro]
pub fn jagaimo(input: TS) -> TS {
    // panic!("{:#?}", input);
    let mut cs: CommandStack = parse_macro_input!(input);

    // TODO auto generate help toml file
    // user only has to input values
    // if let Ok(false) = std::fs::exists("pleh.toml") {
    //     std::fs::File::create_new("pleh.toml").unwrap();
    //     panic!("jagaimo panic");
    // }

    // NOTE print commands
    // let rules = cs.rules_ref();
    // for c in rules.commands() {
    //     println!("{}", c);
    // }

    // NOTE tokenizes the commands and injects aliases to tokens
    let mut tts = TypeTreeStack::from_cmd_stack(cs);
    // println!("{:#?}", cmds);

    // NOTE type tree generation

    tts.generate_type_tree().into()

    // TODO will have to make the type tree generation happen from the tokenized commands
    // since both help and parse need the aliases that are injected into the tokenized command
    // and both features are based upon the type tree
    // so type tree must have the aliases

    // NOTE once the type tree is done
    // i can make the parse fn from the tokens

    // read toml help file
    // extract_from_tables();

    // quote! {}.into()
}
