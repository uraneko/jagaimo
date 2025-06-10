use proc_macro::TokenStream;
use quote::quote;
use syn::parse_derive_input;

mod read;

#[proc_macro_derive(Help, attributes(is_root))]
pub fn derive(input: TokenStream) -> TokenStream {
    input
}
