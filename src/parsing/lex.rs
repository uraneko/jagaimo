use super::parse::Syntax;
use std::str::FromStr;

pub trait Lex {
    fn lex<T: Iterator<Item = String> + std::fmt::Debug>(
        input: T,
    ) -> impl Iterator<Item = Token> + std::fmt::Debug;

    /// allow some literal values [a, b] to be treaded as Bool tokens
    /// instead of their default token kind
    fn overload_bool(value: &str) -> Option<bool> {
        None
    }

    /// allow some literal value x to be treaded as a flag instead of
    /// its default token kind
    fn overload_flag(value: &str) -> Option<Flag> {
        None
    }

    /// specifies what syntaxes are allowed for flags
    /// e.g., --max-depth <- DoubleHyphen is allowed
    /// but, -m <- single hyphen Lowercase is not allowed
    fn flag_syntax() -> Vec<Syntax> {
        vec![]
    }

    /// specifies what syntax is allowed for cli command words
    /// e.g., gh repo list <- repo is the scope
    /// and list is the command <- list is of hyphenless syntax
    fn cmd_syntax() -> Vec<Syntax> {
        vec![]
    }

    /// specifies what syntax is allowed for cli command words
    /// e.g., gh repo list <- repo is the scope
    /// of hyphenless syntax
    fn scope_syntax() -> Vec<Syntax> {
        vec![]
    }
}

/// possible command arg token value by kind
#[derive(Debug)]
pub enum Token {
    Str(String),
    Int(i64),
    Bool(bool),
    Float(f64),
    Flag(Flag),
}

#[derive(Debug)]
pub struct Flag {
    pub(crate) prefix: Prefix,
    pub(crate) value: String,
}

impl Flag {
    fn value(&self) -> String {
        self.value.clone()
    }

    fn value_ref(&self) -> &str {
        &self.value
    }
}

// TODO error handling in these fns
impl Token {
    fn int(value: String) -> Self {
        Self::Int(value.parse::<i64>().unwrap())
    }

    fn float(value: String) -> Self {
        Self::Float(value.parse::<f64>().unwrap())
    }

    fn str(value: String) -> Self {
        Self::Str(value)
    }

    fn bool(value: String) -> Self {
        Self::Bool(value.parse::<bool>().unwrap())
    }

    fn flag(value: String) -> Self {
        Self::Flag(value.parse::<Flag>().unwrap())
    }
}

#[derive(Debug)]
pub enum TokenError {
    CouldNotTokenizeFlagFromStr,
}

impl FromStr for Flag {
    type Err = TokenError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("--") {
            Ok(Flag {
                prefix: Prefix::DoubleHyphen,
                value: s[2..].to_string(),
            })
        } else if s.starts_with('-') {
            Ok(Flag {
                prefix: Prefix::SingleHyphen,
                value: s[1..].to_string(),
            })
        } else {
            Ok(Flag {
                prefix: Prefix::HyphenLess,
                value: s.to_string(),
            })
        }
    }
}

/// prefix of a falg
#[derive(Debug)]
pub enum Prefix {
    SingleHyphen,
    DoubleHyphen,
    HyphenLess,
}

pub struct DefaultLexer;

impl Lex for DefaultLexer {
    fn lex<T: Iterator<Item = String> + std::fmt::Debug>(
        input: T,
    ) -> impl Iterator<Item = Token> + std::fmt::Debug {
        input.into_iter().map(|val| {
            if let Some(f) = Self::overload_flag(&val) {
                Token::Flag(f)
            } else if val.starts_with("--") || val.starts_with('-') {
                Token::flag(val)
            } else if let Some(b) = Self::overload_bool(&val) {
                Token::Bool(b)
            } else if val == "true" || val == "false" {
                Token::bool(val)
            } else if {
                let mut chars = val.chars();
                chars.all(char::is_numeric)
            } {
                Token::int(val)
            } else if {
                let dotless = val.replace('.', "");
                let len = dotless.len();
                dotless.chars().all(char::is_numeric) && len == val.len() - 1
            } {
                Token::float(val)
            } else {
                Token::Str(val)
            }
        })
    }

    fn overload_bool(value: &str) -> Option<bool> {
        match value {
            "yes" => Some(true),
            "no" => Some(false),
            _ => None,
        }
    }

    // fn overload_flag(value: &str) -> Option<Flag> {
    //     match value {
    //         "bind" => Some(Flag {
    //             prefix: Prefix::HyphenLess,
    //             value: value.into(),
    //         }),
    //         "0.0a+dev_98.423" => Some(Flag {
    //             prefix: Prefix::HyphenLess,
    //             value: format!("version {}", value),
    //         }),
    //         _ => None,
    //     }
    // }
}

impl Token {
    pub fn as_u8(&self) -> u8 {
        match self {
            Self::Str(_) => 0,
            Self::Bool(_) => 1,
            Self::Int(_) => 2,
            Self::Float(_) => 4,
            Self::Flag(_) => 8,
        }
    }

    pub fn value(&self) -> String {
        let Self::Flag(f) = self else {
            panic!("{:?}", self)
        };

        f.value()
    }

    pub fn value_ref(&self) -> &str {
        let Self::Flag(f) = self else {
            panic!("{:?}", self)
        };

        f.value_ref()
    }
}
