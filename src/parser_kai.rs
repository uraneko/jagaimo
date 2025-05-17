mod data {
    // enum TToken {
    //     SingleHyphen,
    //     DoubleHyphen,
    //     WhiteSpace,
    //     DoubleQuote,
    //     SingleQuote,
    //     Word(Word),
    //     Lit(Lit),
    // }
    //
    // struct Word {
    //     value: String,
    //     span: usize,
    // }
    //
    // enum Lit {
    //     Float(FloatLit),
    //     Int(IntLit),
    //     Str(StrLit),
    // }

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub enum Token {
        Scope(String),
        Command(String),
        Option(String),
        Argument(String),
    }

    impl Token {
        fn to_u8(&self) -> u8 {
            match self {
                Self::Scope(_) => 0,
                Self::Command(_) => 1,
                Self::Option(_) => 2,
                Self::Argument(_) => 4,
            }
        }

        fn to_string(&self) -> String {
            match self {
                Self::Scope(val) => val.to_string(),
                Self::Command(val) => val.to_string(),
                Self::Option(val) => val.to_string(),
                Self::Argument(val) => val.to_string(),
            }
        }
        fn as_str(&self) -> &str {
            match self {
                Self::Scope(val) => val,
                Self::Command(val) => val,
                Self::Option(val) => val,
                Self::Argument(val) => val,
            }
        }
    }

    impl From<Token> for String {
        fn from(value: Token) -> Self {
            value.to_string()
        }
    }

    impl<'a> From<&'a Token> for &'a str {
        fn from(value: &'a Token) -> Self {
            value.as_str()
        }
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

    #[derive(Debug, Default, Eq, PartialEq)]
    pub struct CLICall {
        pub cmd: CLICommand,
        pub opts: Vec<CLIOption>,
    }
}

pub mod parse {
    use super::WordChecks;
    use super::data::*;

    #[repr(u8)]
    #[derive(Debug, Default)]
    pub enum LexState {
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

    // splits the input string on whitespace char occurrences
    fn chunkify(input: &str) -> impl Iterator<Item = String> + Clone {
        input.split(' ').map(|c| c.to_owned())
    }

    // possible start case
    // scope then command
    // command
    // args <- handled by if statement
    /// tokenizes the input string into a stream of tokens
    pub fn lexer(input: &str) -> Vec<Token> {
        let words = chunkify(input);
        let mut state = LexState::Start;
        // there are no params nor options, only arguments
        if words.clone().all(|w| !w.starts_as_option()) {
            // WARN a second case of this could be that
            // the first word is a command and all subsequent words are its args
            // but that's afaik pretty heretic in cli tools, wont support,
            // cant support either way, since cant tell for sure if 1st arg is a command
            println!("all args");
            words.map(|w| Token::Argument(w)).collect()
        } else {
            println!("do some work on words");
            tokenize_word(words, &mut state, vec![])
        }
    }

    fn tokenize_word<T: Iterator<Item = String>>(
        mut words: T,
        state: &mut LexState,
        mut tokens: Vec<Token>,
    ) -> Vec<Token> {
        let word = words.next();
        if word.is_none() {
            return tokens;
        }
        let word = word.unwrap();

        match *state {
            LexState::Start => {
                let next = words.next();
                if next.is_none() {
                    return tokens;
                }
                let next = next.unwrap();
                if next.starts_as_option() {
                    tokens.extend([Token::Command(word), Token::Option(next)]);
                    *state = LexState::Option;
                } else {
                    tokens.extend([Token::Scope(word), Token::Command(next)]);
                    *state = LexState::Command;
                }
                tokenize_word(words, state, tokens)
            }
            // this never gets reached
            // cuz you cant tell if the 1st tiken is
            // a scope or a command without checking
            // the 2nd token with it
            LexState::Scope => unreachable!(
                "this never gets reached, since you cant tell
if the 1st token is a scope or a command without checking the 2nd token with it"
            ),
            LexState::Command => {
                // error if there are still tokens after the command token then
                // the first token has to be an option
                if !word.starts_as_option() {
                    panic!("maybe return an error");
                }
                tokens.push(Token::Option(word));
                *state = LexState::Option;
                tokenize_word(words, state, tokens)
            }
            LexState::Option | LexState::Argument => {
                if word.starts_as_option() {
                    tokens.push(Token::Option(word));
                    *state = LexState::Option;
                } else {
                    tokens.push(Token::Argument(word));
                    *state = LexState::Argument;
                }
                tokenize_word(words, state, tokens)
            }
        }
    }

    fn parser(tokens: Vec<Token>) -> CLICall {
        let call = CLICall::default();
        if !tokens.is_empty() {
            let tokens = tokens
                .into_iter()
                .filter(|t| t == &Token::Argument("".into()));
            let mut state = LexState::Start;

            return parse_call(tokens, &mut state, call);
        }

        call
    }

    fn parse_call<T: Iterator<Item = Token>>(
        mut tokens: T,
        state: &mut LexState,
        mut call: CLICall,
    ) -> CLICall {
        match state {
            LexState::Start => {}
            LexState::Scope => {}
            LexState::Command => {}
            LexState::Option => {}
            LexState::Argument => {}
        }
        call
    }
}

trait WordChecks {
    fn starts_as_option(&self) -> bool;

    fn contains_space(&self) -> bool;

    fn is_strict_alpha(&self) -> bool;

    fn is_strict_alphanum(&self) -> bool;

    fn is_strict_num(&self) -> bool;
}

impl WordChecks for String {
    fn starts_as_option(&self) -> bool {
        self.starts_with('-') || self.starts_with("--")
    }

    fn contains_space(&self) -> bool {
        self.contains(' ')
    }

    fn is_strict_alpha(&self) -> bool {
        self.chars().all(char::is_alphabetic)
    }

    fn is_strict_num(&self) -> bool {
        self.chars().all(char::is_numeric)
    }

    fn is_strict_alphanum(&self) -> bool {
        self.chars().all(char::is_alphanumeric)
    }
}
