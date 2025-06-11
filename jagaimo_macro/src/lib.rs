use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;

mod input;
mod output;

use input::parse_attrs_rules;

#[proc_macro]
pub fn jagaimo(stream: TokenStream) -> TokenStream {
    // set parser fn that impl Parser trait
    let parser = parse_attrs_rules;
    // parse input using Parser::parse(TokenStream)
    let res = parser.parse(stream);

    // parse input into attrs and rules
    // panic out if error
    let Ok((attrs, rules)) = res else {
        let Err(e) = res else {
            unreachable!("result pattern refutation can not be refuted");
        };
        panic!("failed to parse proc-macro input\n[E] -> {}", e);
    };

    // resolve bare spaces and operations in command rules
    let mut rules = rules.commands_resolution(attrs.root_name(), attrs.ignore_nc());
    // resolve name conflicts of operations

    // print the command rules

    // auto generate additional alias rules if necessary
    rules.alias_generator(attrs.auto_alias());

    let tok_cmds = rules.cmds_tokenizer();

    let q = rules.type_tree_renderer(attrs.root_name(), attrs.derives());

    println!("{}", q);
    q.into()
}

// output
// 1 -> check attrs and if needed make out all aliases
// 2 -> inject aliases into commands
// 3 -> generate type tree
// 4 -> impl help and parse
