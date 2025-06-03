use syn::{Ident, Type};

use crate::input::{AliasRule, AliasScope, CommandRule, Flag};

macro_rules! aliased_token {
    ('f', $t: ident, $a: ident) => {
        AliasedToken::Flag {
            token: $t,
            alias: Some($a),
        }
    };

    ($s: literal, $t: ident, $a: ident) => {
        match $s {
            's' => AliasedToken::Space {
                token: $t,
                alias: Some($a),
            },
            'o' => AliasedToken::Operation {
                token: $t,
                alias: Some($a),
            },
            _ => panic!("macro expected 's' or 'o' as first token"),
        }
    };

    ('f', $t: ident) => {
        AliasedToken::Flag {
            token: $t,
            alias: None,
        }
    };

    ($s: literal, $t: ident) => {
        match $s {
            's' => AliasedToken::Space {
                token: $t,
                alias: None,
            },
            'o' => AliasedToken::Operation {
                token: $t,
                alias: None,
            },
            _ => panic!("macro expected 's', 'o' or 'f' as first token"),
        }
    };

    ($p: ident) => {
        AliasedToken::Params($p)
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct AliasLookup<'a> {
    cmd: &'a CommandRule,
    alias: &'a [AliasRule],
}

impl<'a, 'b> AliasLookup<'b>
where
    'a: 'b,
{
    // creates a new AliasLookup from the passed command rule and slice of alias rules
    pub fn new(cmd: &'a CommandRule, alias: &'a [AliasRule]) -> Self {
        Self { cmd, alias }
    }

    // returns self.cmd's space alias value if it exists
    fn space_aliased(&self) -> Option<AliasedToken<'b>> {
        if let Some(space) = self.cmd.space() {
            let atok = self
                .alias
                .into_iter()
                .find(|al| al.scope() == &AliasScope::S && al.token() == space);
            if atok.is_some() {
                return atok.map(|al| {
                    let al = al.alias();
                    aliased_token!('s', space, al)
                });
            }

            return Some(aliased_token!('s', space));
        }

        None
    }

    // returns self.cmd's op alias value if it exists
    fn op_aliased(&self) -> Option<AliasedToken<'b>> {
        if let Some(op) = self.cmd.op() {
            let atok = self
                .alias
                .into_iter()
                .find(|al| al.scope() == &AliasScope::O && al.token() == op);

            if atok.is_some() {
                return atok.map(|al| {
                    let al = al.alias();
                    aliased_token!('o', op, al)
                });
            }

            return Some(aliased_token!('o', op));
        }

        None
    }

    // returns self.cmd's flags alias values if at least one exists
    fn flags_aliased(&self) -> Option<Vec<AliasedToken<'b>>> {
        if let Some(flags) = self.cmd.flags() {
            return Some(
                flags
                    .into_iter()
                    .map(|f| {
                        let i = f.ident();
                        let al = self
                            .alias
                            .into_iter()
                            .find(|al| al.scope() == &AliasScope::F && i == al.token());

                        if al.is_some() {
                            let a = al.unwrap().alias();
                            aliased_token!('f', f, a)
                        } else {
                            aliased_token!('f', f)
                        }
                    })
                    .collect::<Vec<AliasedToken>>(),
            );
        }

        None
    }

    pub fn lookup(self) -> TokenizedCommand<'b> {
        let mut cmd = TokenizedCommand::default();
        cmd.space = self.space_aliased();
        cmd.op = self.op_aliased();
        cmd.flags = self.flags_aliased();
        cmd.params = self.cmd.params().map(|p| aliased_token!(p));

        cmd
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AliasedToken<'a> {
    Space {
        token: &'a Ident,
        alias: Option<&'a Ident>,
    },
    Operation {
        token: &'a Ident,
        alias: Option<&'a Ident>,
    },
    Flag {
        token: &'a Flag,
        alias: Option<&'a Ident>,
    },
    // this is not really an aliased token
    // but it is needed to be able to vectorize the tokenized command into a vec of tokens
    Params(&'a Type),
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TokenizedCommand<'a> {
    space: Option<AliasedToken<'a>>,
    op: Option<AliasedToken<'a>>,
    flags: Option<Vec<AliasedToken<'a>>>,
    params: Option<AliasedToken<'a>>,
}

impl<'a> From<AliasLookup<'a>> for TokenizedCommand<'a> {
    fn from(lookup: AliasLookup<'a>) -> Self {
        lookup.lookup()
    }
}

impl<'a> From<TokenizedCommand<'a>> for Vec<AliasedToken<'a>> {
    fn from(value: TokenizedCommand<'a>) -> Self {
        let mut v = vec![];
        if let Some(space) = value.space {
            v.push(space);
        }
        if let Some(op) = value.op {
            v.push(op);
        }
        if let Some(flags) = value.flags {
            v.extend(flags);
        }
        if let Some(params) = value.params {
            v.push(params);
        }

        v
    }
}

impl std::fmt::Display for AliasedToken<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use quote::ToTokens;

        write!(
            f,
            "{}",
            match self {
                Self::Space { token, alias } | Self::Operation { token, alias } => format!(
                    "{}{}",
                    token,
                    if let Some(a) = alias {
                        format!("({})", a)
                    } else {
                        "".into()
                    }
                ),
                Self::Flag { token, alias } => format!(
                    "{}{}",
                    token,
                    if let Some(a) = alias {
                        format!("({})", a)
                    } else {
                        "".into()
                    }
                ),
                Self::Params(ty) => format!("{}", ty.to_token_stream().to_string()),
            }
        )
    }
}

impl std::fmt::Display for TokenizedCommand<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "S:{} O:{} F:{:?} P:{}",
            if let Some(space) = &self.space {
                format!("{}", space)
            } else {
                "".into()
            },
            if let Some(op) = &self.op {
                format!("{}", op)
            } else {
                "".into()
            },
            if let Some(flags) = &self.flags {
                flags
                    .into_iter()
                    .map(|f| format!("{}", f))
                    .fold(String::new(), |acc, s| acc + " " + &s)
            } else {
                "".into()
            },
            if let Some(params) = &self.params {
                format!("{}", params)
            } else {
                "".into()
            }
        )
    }
}
