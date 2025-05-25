use proc_macro::TokenStream as TS;
use proc_macro2::TokenStream as TS2;
use quote::ToTokens;
use quote::quote;
use syn::parse_macro_input;

mod engine;
use engine::{CommandTree, Unprocessed};

mod read_manifest;

mod parser;

#[proc_macro]
pub fn jagaimo(input: TS) -> TS {
    // panic!("{:#?}", input);
    let ct: CommandTree<Unprocessed> = parse_macro_input!(input);
    println!("{:#?}", ct);

    quote! {}.into()
}
