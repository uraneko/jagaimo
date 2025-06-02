use crate::parse::alias_token;
use crate::parse::flags::Flag;
use crate::parse::{AliasToken, Aliased, Aliases, CommandRule, CommandStack, Rules};
use syn::{Ident, Type};

impl CommandStack {
    pub fn tokenize_commands(&mut self) -> Vec<TokenizedCommand> {
        // remove command rule duplicates
        self.rules_mut().dedup_commands();
        self.resolve_aliases();
        let cmds = self.take_rules().take_commands();
        let als = self.take_aliases().into_aliases();

        cmds.into_iter()
            .map(|c| (c, als.as_slice()).into())
            // .inspect(|c| println!("\n\n{:?}", c))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TokenizedCommand {
    space: Option<CommandToken>,
    op: Option<CommandToken>,
    flags: Option<Vec<CommandToken>>,
    param: Option<CommandToken>,
}

impl FromIterator<CommandToken> for TokenizedCommand {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = CommandToken>,
    {
        let mut tc = TokenizedCommand::default();
        let mut iter = iter.into_iter();
        while let Some(token) = iter.next() {
            match token {
                CommandToken::Space { .. } => tc.set_space(token),
                CommandToken::Operation { .. } => tc.set_op(token),
                CommandToken::Flag { .. } => tc.push_flag(token),
                CommandToken::Params(_) => tc.set_param(token),
            }
        }

        tc
    }
}

impl From<Vec<CommandToken>> for TokenizedCommand {
    fn from(seq: Vec<CommandToken>) -> Self {
        seq.into_iter().collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandToken {
    Space { ident: Ident, alias: Option<Ident> },
    Operation { ident: Ident, alias: Option<Ident> },
    Flag { flag: Flag, alias: Option<Ident> },
    Params(Type),
}

impl CommandToken {
    fn is_space(&self) -> bool {
        let Self::Space { .. } = self else {
            return false;
        };

        true
    }

    fn is_op(&self) -> bool {
        let Self::Operation { .. } = self else {
            return false;
        };

        true
    }

    fn is_flag(&self) -> bool {
        let Self::Flag { .. } = self else {
            return false;
        };

        true
    }

    pub fn is_param(&self) -> bool {
        let Self::Params { .. } = self else {
            return false;
        };

        true
    }

    pub fn ident(&self) -> Option<&Ident> {
        match self {
            Self::Space { ident, .. } | Self::Operation { ident, .. } => Some(ident),
            _ => None,
        }
    }

    pub fn alias(&self) -> Option<&Ident> {
        match self {
            Self::Space { alias, .. }
            | Self::Operation { alias, .. }
            | Self::Flag { alias, .. } => alias.as_ref(),
            _ => None,
        }
    }

    pub fn flag(&self) -> Option<&Flag> {
        let Self::Flag { flag, .. } = self else {
            return None;
        };

        Some(flag)
    }

    pub fn ty(&self) -> Option<&Type> {
        let Self::Params(ty) = self else {
            return None;
        };

        Some(ty)
    }

    pub fn set_alias(&mut self, als: Ident) {
        match self {
            Self::Space { alias, .. }
            | Self::Operation { alias, .. }
            | Self::Flag { alias, .. } => *alias = Some(als),
            _ => (),
        }
    }

    pub fn new_space(ident: Ident) -> Self {
        Self::Space { ident, alias: None }
    }

    pub fn new_op(ident: Ident) -> Self {
        Self::Operation { ident, alias: None }
    }
}

impl TokenizedCommand {
    pub fn set_space(&mut self, s: CommandToken) {
        if !s.is_space() {
            return;
        }

        self.space = Some(s);
    }

    pub fn set_op(&mut self, o: CommandToken) {
        if !o.is_op() {
            return;
        }

        self.op = Some(o);
    }

    pub fn push_flag(&mut self, f: CommandToken) {
        if !f.is_flag() {
            return;
        }

        match self.flags {
            None => self.flags = Some(vec![f]),
            Some(ref mut flags) => flags.push(f),
        }
    }

    pub fn set_param(&mut self, p: CommandToken) {
        if !p.is_param() {
            return;
        }

        self.param = Some(p);
    }

    // gets the flags directly from the flags field
    // instead of getting the command tokens
    // like the flags method
    pub fn flags2(&self) -> Option<Vec<Flag>> {
        self.flags.clone().map(|flags| {
            flags
                .into_iter()
                .map(|f| f.flag().cloned().unwrap())
                .collect()
        })
    }

    pub fn space_matches(&self, i: &Ident) -> bool {
        let Some(CommandToken::Space { ident, .. }) = self.space() else {
            return false;
        };
        ident == i
    }
}

impl TokenizedCommand {
    pub fn space(&self) -> Option<&CommandToken> {
        self.space.as_ref()
    }

    pub fn op(&self) -> Option<&CommandToken> {
        self.op.as_ref()
    }

    pub fn flags(&self) -> Option<&[CommandToken]> {
        self.flags.as_ref().map(|v| v.as_slice())
    }

    pub fn params(&self) -> Option<&CommandToken> {
        self.param.as_ref()
    }
}

impl From<(CommandRule, &[Aliased])> for TokenizedCommand {
    fn from(value: (CommandRule, &[Aliased])) -> Self {
        let cr = value.0;
        let als = value.1;
        let (spc, op, flgs) = (
            cr.find_space_alias(als),
            cr.find_op_alias(als),
            cr.find_flags_aliases(als),
        );

        let mut tc = Self::default();

        if spc.is_some() {
            tc.space = Some(CommandToken::Space {
                ident: cr.space().unwrap().clone(),
                alias: spc.map(|a| a.clone()),
            });
        };

        if op.is_some() {
            tc.op = Some(CommandToken::Operation {
                ident: cr.op().unwrap().clone(),
                alias: op.map(|a| a.clone()),
            });
        };

        if let Some(p) = cr.params() {
            tc.param = Some(CommandToken::Params(p.clone()));
        }

        println!("{}\n", cr);
        if let Some(mut flgs) = flgs {
            let flags = {
                tc.flags = Some(vec![]);

                tc.flags.as_mut().unwrap()
            };
            while let Some(f) = flgs.next() {
                if let Some(flag) = f {
                    flags.push(flag)
                }
            }
        }

        tc
    }
}
