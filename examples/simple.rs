use nikujaga::parser;

fn main() {
    let mut args = std::env::args();
    args.next();
    println!("\nargs: \n{:#?}", args);

    let tokens = parser::lex(args, vec![], ("".into(), false));
    println!("\ntokens: \n{:#?}", tokens);

    // let call = parser::parse(tokens.into_iter(), parser::CLICall::default());
    // println!("\ncall: \n{:#?}", call);
}
