use proc_macro::TokenStream;
use quote::ToTokens;
use quote::quote;
use syn::{Attribute, Expr, ExprLit, Meta};
use syn::{DeriveInput, parse_macro_input};

mod attrs;
mod help;
mod read;
mod styled;

#[proc_macro_derive(Help, attributes(root, space, at))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse_macro_input!(input);
    let attrs = attrs::HelpAttrs::new(ast.attrs);
    println!("{:?}", attrs);

    quote! {}.into()
}

// NOTE
// howto macro
// 1/ get attrs... done
// 2/ read help file... done
// 4/ match on type scope using attrs -> root or space or op... done
// 5/ generate necessary excavator
// 6/ use excavator to get relevant help data from read file data... done
// 7/ use relevant data to impl Help for matched type scope
// 8/ done

