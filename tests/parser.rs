use nikujaga::parser::{lex, parse, tokenize};
use nikujaga::parser::{CLICall, Token};

#[test]
#[ignore = "this doesn't necessitate a test"]
fn test_tokenize() {
    let s = "a123 random t#est @string !here \"{ }\
        as dask";

    assert_eq!(
        vec!["a123", "random", "t#est", "@string", "!here", "\"{", "}as", "dask"],
        tokenize(s)
    );
}

#[test]
fn test_lex() {
    let cmd = "git config --global user.email";
    let mut words = tokenize(cmd).into_iter();
    // get rid of the program name
    words.next();

    let tokens = vec![
        Token::Cmd("config".into()),
        Token::Opt("--global".into()),
        Token::Arg("user.email".into()),
    ];

    lex(words.into_iter(), vec![], ("".into(), false))
        .into_iter()
        .zip(tokens)
        .for_each(|(a, b)| assert_eq!(a, b))
}

#[test]
fn test_parse() {
    let tokens = vec![
        Token::Cmd("config".into()),
        Token::Opt("--global".into()),
        Token::Arg("user.email".into()),
    ];

    let call = CLICall::default()
        .cmd("config")
        .opt_with_arg("--global", "user.email");

    assert_eq!(call, parse(tokens.into_iter(), CLICall::default()).unwrap());
}
