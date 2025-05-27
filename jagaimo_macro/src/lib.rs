use proc_macro::TokenStream as TS;
use proc_macro2::TokenStream as TS2;
use quote::ToTokens;
use quote::quote;
use syn::parse_macro_input;

// DOCS
//               command
//  _____________________________________
// |                                     |
// executable realm operation flags params
// |                        | |          |
// ---------------------------------------
//           call              context

mod engine;
mod resolve_crate;
mod traits;

use engine::{CommandTree, RuleBook};

#[proc_macro]
pub fn jagaimo(input: TS) -> TS {
    // panic!("{:#?}", input);
    let ct: CommandTree = parse_macro_input!(input);
    println!("{:#?}", ct);
    let rb = ct.rules();
    let realms = rb.generate_realms();

    quote! {
        #(#realms)*
    }
    .into()
}
