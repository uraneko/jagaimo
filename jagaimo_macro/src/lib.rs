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

// BUG
// error: expected identifier, found `,`
//   --> jagaimo/src/lib.rs:3:1
//    |
// 3  | / jagaimo! {
// 4  | |     #[
// 5  | |         no_help,
// 6  | |         no_version,
// ...  |
// 29 | | }
//    | | ^
//    | | |
//    | |_expected identifier
//    |   while parsing this struct
//    |
//    = note: this error originates in the macro `jagaimo` (in Nightly builds, run with -Z macro-backtrace for more info)
//
// the macro generated structs look normal upon expansion and they compile safely in a playground
// this is not a macro parsing problem as the macro parses and generates code as intended
// this is a compiler parsing problem. the compiler cant accepet some of the macro input; it
// doesn't lex into proper rust tokens

#[proc_macro]
pub fn jagaimo(input: TS) -> TS {
    // panic!("{:#?}", input);
    let mut ct: CommandStack = parse_macro_input!(input);

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

    // quote! {}.into()
}
