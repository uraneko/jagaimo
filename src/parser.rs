use std::collections::HashMap;
use std::mem::discriminant;

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum Token {
    Scope(String) = 0,
    Cmd(String) = 1,
    Opt(String) = 2,
    Arg(String) = 4,
}

impl Token {
    fn to_u8(&self) -> u8 {
        match self {
            Self::Scope(val) => 0,
            Self::Cmd(val) => 1,
            Self::Opt(val) => 2,
            Self::Arg(val) => 4,
        }
    }
}

impl Token {
    fn to_string(&self) -> String {
        match self {
            Self::Scope(val) => val.to_string(),
            Self::Cmd(val) => val.to_string(),
            Self::Opt(val) => val.to_string(),
            Self::Arg(val) => val.to_string(),
        }
    }
    fn as_str(&self) -> &str {
        match self {
            Self::Scope(ref val) => val,
            Self::Cmd(ref val) => val,
            Self::Opt(ref val) => val,
            Self::Arg(ref val) => val,
        }
    }
}

impl From<Token> for String {
    fn from(value: Token) -> Self {
        value.to_string()
    }
}

#[derive(Debug)]
pub struct CLICall {
    cmd: CLICommand,
    opts: Vec<CLIOption>,
}

impl Default for CLICall {
    fn default() -> Self {
        Self {
            cmd: CLICommand::Command("".into()),
            opts: vec![],
        }
    }
}

#[derive(Debug)]
pub enum CLICommand {
    Command(String),
    ScopedCommand { scope: String, cmd: String },
}

#[derive(Debug)]
pub enum CLIOption {
    Option(String),
    OptionWithArg { opt: String, arg: String },
}

pub fn tokenize(input: &str) -> Vec<String> {
    input.split(' ').map(|s| s.to_owned()).collect()
}

//       cli scope  cmd      opt     opt            arg
// e.g., git config reset --hard --config-file "path/to/config/file"
pub fn lex<T>(mut words: T, mut tokens: Vec<Token>, mut arg: (String, bool)) -> Vec<Token>
where
    T: Iterator<Item = String>,
{
    let word = words.next();
    if word.is_none() {
        println!("{}", line!());
        return tokens;
    }
    let word = word.unwrap();

    match tokens.last() {
        // first token
        // scope or cmd
        None => {
            let next = words.next();
            if next.is_none() {
                println!("{}", line!());
                return vec![Token::Cmd(word)];
            }
            let next = next.unwrap();
            if next.starts_with("--") || next.starts_with('-') {
                tokens.extend([Token::Cmd(word), Token::Opt(next)]);
                lex(words, tokens, arg)
            } else {
                tokens.extend([Token::Scope(word), Token::Cmd(next)]);
                lex(words, tokens, arg)
            }
        }
        // anything
        Some(tok) => {
            match tok.to_u8() {
                // the last token was a scope
                0 => {
                    tokens.push(Token::Cmd(word));

                    // return lex(words, tokens, arg);
                }
                // last token was a cmd
                1 => {
                    tokens.push(Token::Opt(word));

                    // return lex(words, tokens, arg);
                }
                // last token was an opt
                // arg start or next opt
                2 if arg.0.is_empty() => {
                    match word.starts_with("--") || word.starts_with('-') {
                        true => {
                            tokens.push(Token::Opt(word));
                        }
                        false => {
                            arg.1 = if word.starts_with('"') { true } else { false };
                            arg.0.push_str(&word);
                        }
                    }

                    // return lex(words, tokens, arg);
                }
                // last token was an arg chunk
                // arg continuation
                2 if !arg.0.is_empty() => {
                    match [arg.1, word.ends_with('"'), word.ends_with("\"")] {
                        // arg value is enclosed by dbbl quotes and done
                        [true, true, false] => {
                            arg.0.push(' ');
                            arg.0.push_str(&word);
                            tokens.push(Token::Arg(arg.0.drain(..).collect()));
                            // just return after the match statement
                            // return lex(words, tokens, arg);
                        }
                        // enclosed but not done yet
                        // [true, ..]
                        [true, false, _] | [true, true, true] => {
                            arg.0.push(' ');
                            arg.0.push_str(&word);

                            // return lex(words, tokens, arg);
                        }
                        // not enclosed by dbl quotes
                        [false, ..] => {
                            // BUG
                            // after the lexer takes an arg that is not dbl quote enclosed
                            // it gets stuck here on all iterations of the
                            // recursion
                            // until it feeds all the words to the arg string and quits because
                            // no more words can be found
                            let next = words.next();
                            if next.is_none() {
                                println!("{}, {}", line!(), arg.0);
                                return tokens;
                            }
                            let next = next.unwrap();
                            arg.0.push(' ');
                            arg.0.push_str(&word);
                            if next.starts_with("--") || next.starts_with('-') {
                                tokens.extend([
                                    Token::Arg(arg.0.drain(..).collect()),
                                    Token::Opt(word),
                                ]);
                            } else {
                                arg.0.push(' ');
                                arg.0.push_str(&next);
                            }

                            // return lex(words, tokens, arg);
                        }
                    }
                }
                e => unreachable!("value {} not set for any variant of Token enum", e),
            }
            lex(words, tokens, arg)
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    FoundScopeWithNoCommand,
    GotUnexpectedCommandAmongstOptions,
    GotUnexpectedScopeAmongstOptions,
    ArgTokenHasEscapedTheOptionNet,
}

pub fn parse<T>(mut tokens: T, mut call: CLICall) -> Result<CLICall, ParseError>
where
    T: Iterator<Item = Token>,
{
    let tok = tokens.next();

    if tok.is_none() {
        return Ok(call);
    }
    let tok = tok.unwrap();

    match tok {
        Token::Scope(val) => {
            let next = tokens.next();
            if next.is_none() || next.as_ref().unwrap().to_u8() != 1 {
                return Err(ParseError::FoundScopeWithNoCommand);
            }
            let next = next.unwrap();
            call.cmd = CLICommand::ScopedCommand {
                scope: next.into(),
                cmd: val,
            };
        }
        Token::Cmd(val) => call.cmd = CLICommand::Command(val),
        Token::Opt(val) => {
            let next = tokens.next();
            if next.is_none() {
                call.opts.push(CLIOption::Option(val));

                return Ok(call);
            }
            let next = next.unwrap();
            match &next.to_u8() {
                0 => return Err(ParseError::GotUnexpectedScopeAmongstOptions),
                1 => return Err(ParseError::GotUnexpectedCommandAmongstOptions),
                2 => call
                    .opts
                    .extend([CLIOption::Option(val), CLIOption::Option(next.into())]),
                4 => call.opts.push(CLIOption::OptionWithArg {
                    opt: val,
                    arg: next.into(),
                }),
                val => unreachable!("no such variant value: {}", val),
            }
        }
        Token::Arg(val) => return Err(ParseError::ArgTokenHasEscapedTheOptionNet),
    }

    parse(tokens, call)
}
