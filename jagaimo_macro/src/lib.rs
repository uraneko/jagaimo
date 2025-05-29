use proc_macro::TokenStream as TS;
use proc_macro2::TokenStream as TS2;
use quote::ToTokens;
use quote::quote;
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
    ct.resolve_aliases();
    println!("{:#?}", ct.aliases_ref());
    // let rb = ct.rules();
    // for c in rb.commands() {
    //     println!("{}", c);
    // }

    quote! {}.into()
}
