use nikujaga::parser_kai2::{CommandParser, DefaultParser};
fn main() {
    // "build --release --no-default-features --features \"nightly binary encoding_decoding\"";
    let mut args = std::env::args();
    args.next();

    println!("{:?}", args);
    println!("{:#?}", DefaultParser::lex(args));
}
