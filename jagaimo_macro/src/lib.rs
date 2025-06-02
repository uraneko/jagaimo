use proc_macro::TokenStream;
use syn::parse::Parser;

mod input;
mod resolve_crate;

use input::parse_attrs_rules;

#[proc_macro]
pub fn jagaimo(stream: TokenStream) -> TokenStream {
    let parser = parse_attrs_rules;
    let res = parser.parse(stream);

    let Ok((attrs, rules)) = res else {
        let Err(e) = res else {
            unreachable!("result pattern refutation can not be refuted");
        };
        panic!("failed to parse proc-macro input\n[E] -> {}", e);
    };
    println!("{:#?}", attrs);

    quote::quote! {}.into()
}
