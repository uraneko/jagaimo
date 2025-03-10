use nikujaga::parser::{lex, parse, tokenize};
use nikujaga::parser::{CLICall, Token};

#[test]
// #[should_panic]
fn break_on_successive_args1() {
    // cli command: \$> cat a.txt b.txt
    // pass command words directly to the lexer
    // lex the words into proper tokens
    let tokens = lex(
        ["a.txt", "b.txt"].into_iter().map(|w| w.to_string()),
        vec![],
        (String::new(), false),
    );
    println!("got {} tokens", tokens.len());

    // parse the tokens into a CLICall
    let call = parse(tokens.into_iter(), CLICall::default());
    println!("the parsed command is: {:?}", call);
}

#[test]
// #[should_panic]
fn break_on_successive_args2() {
    // cli command: \$> git config --global user.email aaa@bbb.ccc
    // tokenize the cli command into words
    let mut words = tokenize("git config --global user.email aaa@bbb.ccc").into_iter();
    // ignore the cli program name word; `git`
    words.next();
    println!("tokenized command into words: {:?}", words);

    // lex the words into proper tokens
    let tokens = lex(words.into_iter(), vec![], (String::new(), false));
    println!("got {} tokens", tokens.len());

    // parse the tokens into a CLICall
    let call = parse(tokens.into_iter(), CLICall::default());
    println!("the parsed command is: {:?}", call);
}
