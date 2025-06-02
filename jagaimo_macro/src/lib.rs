use proc_macro::TokenStream;
use syn::parse_macro_input;

mod input;
mod resolve_crate;

#[proc_macro]
pub fn jagaimo(stream: TokenStream) -> TokenStream {
    stream
}
