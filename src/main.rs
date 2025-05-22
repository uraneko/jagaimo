use nikujaga::parsing::*;
use nikujaga::parsing::{lex::Lex, parse::Parse};

fn main() {
    // "build --release --no-default-features --features \"nightly binary encoding_decoding\"";
    let args = args();
    println!("{:?}", args);

    let tokens: Vec<lex::Token> = lex::DefaultLexer::lex(args).collect();
    println!("{:#?}", tokens);
    let parsed = parse::DefaultParser::parse(tokens.into_iter());
    println!("{:#?}", parsed);
}
