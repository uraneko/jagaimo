use super::lex::Token;
use std::collections::HashMap;

pub trait Parse {
    type Output;

    fn parse<T: std::iter::Iterator<Item = Token>>(tokens: T) -> Self::Output;

    fn scope_is_default() -> bool {
        false
    }

    fn action_is_default() -> bool {
        true
    }

    // all values that return true
    // will always be treated as scopes
    // when they are found to be the first token
    fn override_scope(value: &str) -> bool {
        false
    }

    // TODO need patterns
    // like
    // if pat1 token and pat3 token then case1
    // else if pat2 token and pat3 token then case2

    fn override_flag(value: &str) -> bool {
        // match value {
        // "global" => true,
        // _ => false
        // }
        false
    }

    fn custom_command(tokens: Vec<Token>) -> Option<Command> {
        match &tokens[..] {
            // when the -v flag is found
            [
                Token::Str(s),
                Token::Flag(super::lex::Flag {
                    prefix: super::lex::Prefix::SingleHyphen,
                    value: v,
                }),
            ] if s == "remote" && v == "v" => Some(Command {
                scope: Some("remote".into()),
                action: None,
                args: None,
                flags: Some(vec!["v".into()]),
                opts: None,
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Default)]
pub struct Command {
    scope: Option<String>,
    action: Option<String>,
    args: Option<Vec<Arg>>,
    flags: Option<Vec<String>>,
    opts: Option<HashMap<String, Arg>>,
}

#[derive(Debug)]
enum CommandToken {
    Option,
    Flag,
    Scope,
    Action,
    Arg,
}

/// possible arg types
#[derive(Debug)]
pub enum Arg {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    // Date,
    // Phone,
    // Email,
    // URL,
    List(Vec<Arg>),
}

impl From<Token> for Arg {
    fn from(value: Token) -> Self {
        match value {
            Token::Str(s) => Self::Str(s),
            Token::Float(f) => Self::Float(f),
            Token::Int(i) => Self::Int(i),
            Token::Bool(b) => Self::Bool(b),
            _ => panic!("didnt expect flag variant"),
        }
    }
}

#[derive(Debug)]
pub enum Syntax {
    /// -a
    SingleHyphenLowerCase,
    /// -A
    SingleHyphenUpperCase,
    /// --yada-yada
    DoubleHyphen,
    /// this-and-that
    HyphenLess,
}

pub struct DefaultParser;

pub trait CommandModifier {
    fn modify_command(cmd: &mut Command);
}

#[repr(u8)]
#[derive(Debug, Default, PartialEq)]
enum Last {
    #[default]
    None,
    Flag,
    Arg,
    Act,
    Scope,
}

impl Parse for DefaultParser {
    type Output = Command;

    fn parse<T: Iterator<Item = Token>>(mut tokens: T) -> Self::Output {
        let mut cmd = Command::default();
        cmd.action = tokens.next().map(|t| {
            let Token::Str(s) = t else {
                panic!("first token wasnt a string")
            };
            s
        });
        let mut last = Last::None;
        'w0: while let Some(token) = tokens.next() {
            match token.as_u8() {
                // str
                0 | 1 | 2 | 4 => {
                    if last == Last::None {
                        if cmd.args.is_none() {
                            cmd.args = Some(vec![token.into()]);
                        } else {
                            cmd.args.as_mut().unwrap().push(token.into());
                        }
                    } else if Last::Flag == last {
                        let mut args: Vec<Arg> = vec![token.into()];
                        while let Some(arg) = tokens.next() {
                            if let Token::Flag(_) = arg {
                                if cmd.flags.is_none() {
                                    cmd.flags = Some(vec![arg.value()]);
                                } else {
                                    cmd.flags.as_mut().unwrap().push(arg.value());
                                }
                                last = Last::Flag;

                                continue 'w0;
                            } else {
                                args.push(arg.into());
                            }
                        }
                        last = Last::Arg;

                        if cmd.opts.is_none() {
                            cmd.opts = Some(HashMap::from([(
                                cmd.flags.as_mut().unwrap().pop().unwrap(),
                                Arg::List(args),
                            )]));
                        } else {
                            cmd.opts.as_mut().unwrap().insert(
                                cmd.flags.as_mut().unwrap().pop().unwrap(),
                                Arg::List(args),
                            );
                        }
                    }
                }
                // flag
                8 => {
                    if Self::override_flag(&token.value_ref()) {
                        if cmd.flags.is_none() {
                            cmd.flags = Some(vec![token.value()]);
                        } else {
                            cmd.flags.as_mut().unwrap().push(token.value());
                        }
                    } else if let Some(opt) = tokens.next() {
                        let mut args: Vec<Arg> = vec![];
                        while let Some(arg) = tokens.next() {
                            if let Token::Flag(_) = arg {
                                if cmd.flags.is_none() {
                                    cmd.flags = Some(vec![opt.value(), arg.value()]);
                                } else {
                                    cmd.flags
                                        .as_mut()
                                        .unwrap()
                                        // BUG wrong behavior
                                        // the seoncd item is not necessarily a flag
                                        // could be an opt
                                        // nevermind just popped the last flag when needed
                                        .extend([opt.value(), arg.value()]);
                                }
                                last = Last::Flag;

                                continue 'w0;
                            } else {
                                args.push(arg.into());
                            }
                        }
                        last = Last::Arg;

                        if args.is_empty() {
                            if cmd.flags.is_none() {
                                cmd.flags = Some(vec![opt.value()]);
                            } else {
                                cmd.flags.as_mut().unwrap().push(opt.value());
                            }
                        } else {
                            if cmd.opts.is_none() {
                                cmd.opts = Some(HashMap::from([(opt.value(), Arg::List(args))]));
                            } else {
                                cmd.opts
                                    .as_mut()
                                    .unwrap()
                                    .insert(opt.value(), Arg::List(args));
                            }
                        }
                    }
                }
                _ => unreachable!("matching on enum variants' u8 repr"),
            }
        }

        panic!()
    }

    fn override_scope(value: &str) -> bool {
        match value {
            // this enforces that when remote appears as the first token
            // it is always considered a scope
            "remote" => true,
            "stash" => true,
            _ => false,
        }
    }
}
