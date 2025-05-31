use proc_macro::TokenStream as TS;
use proc_macro2::Span;
use proc_macro2::TokenStream as TS2;
use quote::ToTokens;
use quote::quote;
use syn::Ident;
use syn::parse_macro_input;

// DOCS
//                  command
//  _____________________________________
// |                                     |
// executable space operation flags params
// |                        | |          |
// ---------------------------------------
//             scope            context

mod generate;
mod parse;
mod process;
mod resolve_crate;

use parse::{CommandStack, Rules};

#[proc_macro]
pub fn jagaimo(input: TS) -> TS {
    // panic!("{:#?}", input);
    let mut ct: CommandStack = parse_macro_input!(input);
    println!("000");

    // NOTE print commands
    // let rules = ct.rules_ref();
    // for c in rules.commands() {
    //     println!("{}", c);
    // }

    // NOTE type tree generation
    let name = ct.attrs().root_name();
    ct.rules_ref()
        .generate_type_tree(&Ident::new(name, Span::call_site()))
        .into()

    // NOTE tokenizes the commands and injects aliases to tokens
    // ct.resolve_aliases();
    // let cmds = ct.tokenize_commands();
    // println!("{:#?}", cmds);

    // NOTE once the type tree is done
    // i can make the parse fn from the tokens
}
