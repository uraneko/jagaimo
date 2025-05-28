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

mod engine;
mod resolve_crate;
mod traits;

use engine::{CommandTree, RuleBook};

#[proc_macro]
pub fn jagaimo(input: TS) -> TS {
    // panic!("{:#?}", input);
    let ct: CommandTree = parse_macro_input!(input);
    // println!("{:#?}", ct);
    let rb = ct.rules();
    for c in rb.commands() {
        println!("{}", c);
    }

    quote! {}.into()
}
