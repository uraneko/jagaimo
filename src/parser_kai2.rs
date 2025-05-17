#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Flag { value: String, prefix: Hyphen },
    WhiteSpace,
    DoubleQuote,
    Word(String),
    Int(String),
    Float(String),
    Str(String),
    Version(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Hyphen {
    Single,
    Double,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum CLICommand {
    #[default]
    None,
    Command(String),
    ScopedCommand {
        scope: String,
        cmd: String,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CLIOption {
    Param(String),
    OptionWithArg { opt: String, arg: String },
    OptionWithArgs { opt: String, args: Vec<String> },
}

impl CLIOption {
    fn param(s: String) -> Self {
        Self::Param(s)
    }

    fn opt_with_arg(opt: String, arg: String) -> Self {
        Self::OptionWithArg { opt, arg }
    }

    fn opt_with_args(opt: String, args: Vec<String>) -> Self {
        Self::OptionWithArgs { opt, args }
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct CLICall {
    pub cmd: CLICommand,
    pub opts: Vec<CLIOption>,
}

#[repr(u8)]
#[derive(Debug, Default)]
pub enum ParseState {
    // just started lexing
    // 0 words lexed
    // 0 tokens in
    #[default]
    Start = 0,
    // already lexed a scope token
    // what comes next should be a
    // scope or a command
    Scope = 1,
    // already lexed a command token
    // next should be an option or nothing
    Command = 2,
    // after lexing an option token
    // what comes next may be the 1st arg of the option
    // or a differnt option (first option was a param)
    Option = 4,
    // after lexing an argument token
    // what comes after may be a new option
    // or other args of the same option
    Argument = 8,
}

fn tokenize<T: Iterator<Item = String>>(mut words: T, mut tokens: Vec<Token>) -> Vec<Token> {
    let word = words.next();
    if word.is_none() {
        return tokens;
    }
    let word = word.unwrap();
    match word {
        val if val.starts_with("--") => tokens.push(Token::Flag {
            value: val[2..].to_string(),
            prefix: Hyphen::Double,
        }),
        val if val.starts_with('-') => tokens.push(Token::Flag {
            value: val[1..].to_string(),
            prefix: Hyphen::Single,
        }),
        val if val.chars().all(char::is_numeric) => tokens.push(Token::Int(val)),
        val if {
            let valr = val.replace('.', "");
            valr.len() == val.len() - 1 && valr.chars().all(char::is_numeric)
        } =>
        {
            tokens.push(Token::Float(val))
        }
        val if val.starts_with('"') => {
            let mut s = val[1..].to_owned();
            loop {
                let next = words.next();
                if next.is_none() {
                    panic!("todo: handle ill terminated str lit in args");
                }
                let next = next.unwrap();
                s.push(' ');
                if next.ends_with('"') {
                    break s.push_str(next.trim_end_matches('"'));
                }
                s.push_str(&next);
            }
            tokens.push(Token::Str(s));
        }
        val => tokens.push(Token::Word(val)),
    }

    tokenize(words, tokens)
}

pub trait CommandParser {
    type Output;

    fn lex(input: std::env::Args) -> Vec<Token> {
        let mut tokens: Vec<Token> = vec![];

        tokenize(input, tokens)
    }

    fn parse(tokens: Vec<Token>) -> Self::Output;
}

/// the default cli command parser
/// assumes the inputted command to be in the form COMMAND [OPTIONS]
/// e.g., git log --graph --pretty=oneline
///       EXE CMD   OPT      OPT    ARG
pub struct DefaultParser;
impl DefaultParser {
    fn parse_recurse<T: Iterator<Item = Token>>(mut tokens: T, mut call: CLICall) -> CLICall {
        let token = tokens.next();
        if token.is_none() {
            return call;
        }
        let token = token.unwrap();

        match token {
            Token::Flag { value, prefix } => {}
            Token::Int(v) | Token::Float(v) | Token::Str(v) => {}
            Token::Word(w) => {}
            tkn => panic!("got unexpected token: {:?}", tkn),
        }

        Self::parse_recurse(tokens, call)
    }
}

/// the scoped command parser
/// assumes the command to be in the form SCOPE COMMAND [OPTIONS]
/// e.g., gh issue create --body "a bug was found  in yadayada, working on it" --title "fix yadayada" --label "bug" --label "test"
pub struct ScopedParser;
/// parser for a simple command that takes only arguments
/// assumed form CMD [ARGS]
/// e.g., cat a.txt b.txt
pub struct SimpleArgsParser;

/// if none of the builtin parsers work for you
/// then you can either
/// implement CommandParser::parse for a CustomParser type of your own
/// or
/// derive an auto implementation for CustomParser
impl CommandParser for DefaultParser {
    type Output = CLICall;

    fn parse(tokens: Vec<Token>) -> Self::Output {
        let tokens = tokens.into_iter();
        let call = CLICall::default();

        Self::parse_recurse(tokens, call)
    }
}

// #[derive(CommandParser)]
// #[option(...)]
// #[command(...)]
// struct CustomParser;
