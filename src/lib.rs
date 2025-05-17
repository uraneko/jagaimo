pub mod lex;
pub mod parse;
pub mod parser;
pub mod parser_kai;
pub mod parser_kai2;
pub use parser::*;

// TODO base developement on already existing cli tools
// for the default parser, aim to support github
// for the macrosm aim to support git
// aim to support cargo, figure out what features should be added for that
// add tests and benches
