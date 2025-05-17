pub mod lex;
pub mod parse;

pub fn args() -> std::env::Args {
    let mut args = std::env::args();
    args.next();

    args
}
