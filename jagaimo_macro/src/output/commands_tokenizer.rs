use syn::{Ident, Type};

use crate::input::{AliasRule, AliasScope, CommandRule, Flag};

macro_rules! aliased_token {
    ('f', $t: ident, $a: ident) => {
        AliasedToken::Flag {
            token: $t,
            alias: Some($a),
        }
    };

    ($s: literal, $t: ident, $a: ident, $n: ident) => {
        match $s {
            's' => AliasedToken::Space {
                token: $t,
                alias: Some($a),
                nameless: $n,
            },
            'o' => AliasedToken::Operation {
                token: $t,
                alias: Some($a),
                nameless: $n,
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

    ($s: literal, $t: ident, $n: ident) => {
        match $s {
            's' => AliasedToken::Space {
                token: $t,
                alias: None,
                nameless: $n,
            },
            'o' => AliasedToken::Operation {
                token: $t,
                alias: None,
                nameless: $n,
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
    fn space_aliased(&self) -> AliasedToken<'b> {
        // WARN altered for the sake expanding
        // if let Some(space) = self.cmd.space() { ...
        let space = self.cmd.space();
        let nameless = !self.cmd.contains_space();
        let atok = self
            .alias
            .into_iter()
            .find(|al| al.scope() == &AliasScope::S && al.token() == space);
        if let Some(al) = atok {
            let al = al.alias();

            return aliased_token!('s', space, al, nameless);
        }

        aliased_token!('s', space, nameless)
    }

    // returns self.cmd's op alias value if it exists
    fn op_aliased(&self) -> AliasedToken<'b> {
        // WARN altered for the sake expanding
        // if let Some(op) = self.cmd.op() { ...
        let op = self.cmd.op();
        let nameless = !self.cmd.contains_op();
        let atok = self
            .alias
            .into_iter()
            .find(|al| al.scope() == &AliasScope::O && al.token() == op);
        if let Some(al) = atok {
            let al = al.alias();

            return aliased_token!('o', op, al, nameless);
        }

        aliased_token!('o', op, nameless)
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
        let mut cmd = TokenizedCommand::new(
            self.space_aliased(),
            self.op_aliased(),
            self.flags_aliased(),
            self.cmd.params().map(|p| aliased_token!(p)),
        );

        cmd
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AliasedToken<'a> {
    Space {
        token: &'a Ident,
        alias: Option<&'a Ident>,
        nameless: bool,
    },

    Operation {
        token: &'a Ident,
        alias: Option<&'a Ident>,
        nameless: bool,
    },

    Flag {
        token: &'a Flag,
        alias: Option<&'a Ident>,
    },

    // this is not really an aliased token
    // but it is needed to be able to vectorize the tokenized command into a vec of tokens
    Params(&'a Type),
}

impl<'a> AliasedToken<'a> {
    pub fn new_op(ident: &'a Ident, nameless: bool) -> Self {
        Self::Operation {
            token: ident,
            alias: None,
            nameless,
        }
    }

    pub fn is_space(&self) -> bool {
        let Self::Space { .. } = self else {
            return false;
        };

        true
    }

    pub fn ident(&self) -> Option<&'a Ident> {
        match self {
            Self::Space { token, .. } | Self::Operation { token, .. } => Some(token),
            _ => None,
        }
    }

    pub fn is_nameless(&self) -> bool {
        match self {
            Self::Space { nameless, .. } | Self::Operation { nameless, .. } => *nameless,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenizedCommand<'a> {
    space: AliasedToken<'a>,
    op: AliasedToken<'a>,
    flags: Option<Vec<AliasedToken<'a>>>,
    params: Option<AliasedToken<'a>>,
}

impl TokenizedCommand<'_> {
    pub fn is_space_op(&self) -> bool {
        !self.space.is_nameless() && !self.op.is_nameless()
    }

    pub fn is_op(&self) -> bool {
        self.space.is_nameless() && !self.op.is_nameless()
    }

    pub fn is_space(&self) -> bool {
        !self.space.is_nameless() && self.op.is_nameless()
    }
}

impl<'a> TokenizedCommand<'a> {
    pub fn new(
        space: AliasedToken<'a>,
        op: AliasedToken<'a>,
        flags: Option<Vec<AliasedToken<'a>>>,
        params: Option<AliasedToken<'a>>,
    ) -> Self {
        Self {
            space,
            op,
            flags,
            params,
        }
    }
    pub fn space(&self) -> AliasedToken<'a> {
        self.space.clone()
    }

    pub fn op(&self) -> AliasedToken<'a> {
        self.op.clone()
    }
}

impl<'a> From<AliasLookup<'a>> for TokenizedCommand<'a> {
    fn from(lookup: AliasLookup<'a>) -> Self {
        lookup.lookup()
    }
}

impl<'a> From<TokenizedCommand<'a>> for Vec<AliasedToken<'a>> {
    fn from(value: TokenizedCommand<'a>) -> Self {
        let mut v = vec![];

        v.push(value.space);

        v.push(value.op);

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
                Self::Space {
                    token,
                    alias,
                    nameless,
                }
                | Self::Operation {
                    token,
                    alias,
                    nameless,
                } =>
                    if *nameless {
                        "".into()
                    } else {
                        format!(
                            "{}{}",
                            token,
                            if let Some(a) = alias {
                                format!("({})", a)
                            } else {
                                "".into()
                            }
                        )
                    },
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
            if !self.space.is_nameless() {
                format!("{}", self.space)
            } else {
                "".into()
            },
            if !self.op.is_nameless() {
                format!("{}", self.op)
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
