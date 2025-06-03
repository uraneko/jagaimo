use proc_macro::TokenStream;
use syn::parse::Parser;

mod input;
mod output;
mod resolve_crate;

use input::parse_attrs_rules;
use output::alias_generation::AliasGenerator;

// BUG
// declaring a bool flag before declaring a param in a command rule
// make the param into the type of the bool flag
// and results in a command without params
//

#[proc_macro]
pub fn jagaimo(stream: TokenStream) -> TokenStream {
    // set parser fn that impl Parser trait
    let parser = parse_attrs_rules;
    // parse input using Parser ::parse(TokenStream)
    let res = parser.parse(stream);

    // parse input into attrs and rules
    // panic out if error
    let Ok((attrs, mut rules)) = res else {
        let Err(e) = res else {
            unreachable!("result pattern refutation can not be refuted");
        };
        panic!("failed to parse proc-macro input\n[E] -> {}", e);
    };
    // print the attributes
    println!("{:#?}", attrs);

    // print the command rules
    for cmd in rules.cmd_ref() {
        println!("{}", cmd);
    }

    // auto generate additional alias rules if necessary
    rules.alias_generator(attrs.auto_alias());

    // print all alias rules
    for al in rules.alias_ref() {
        println!("{}", al);
    }

    println!();

    let tok_cmds = rules.cmds_tokenizer();
    for tcmd in tok_cmds {
        println!("{}\n", tcmd);
    }

    quote::quote! {}.into()
}

// output
// 1 -> check attrs and if needed make out all aliases
// 2 -> inject aliases into commands
// 3 -> generate type tree
// 4 -> impl help and parse
