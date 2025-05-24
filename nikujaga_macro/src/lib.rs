use proc_macro::TokenStream as TS;
use proc_macro2::TokenStream as TS2;
use quote::ToTokens;
use quote::quote;
use syn::parse_macro_input;

mod engine;
use engine::RuleBook;

mod read_manifest;

#[proc_macro]
pub fn nikujaga(input: TS) -> TS {
    // panic!("{:#?}", input);
    let rb: RuleBook = parse_macro_input!(input);
    println!("{:#?}", rb);

    quote! {}.into()
}
