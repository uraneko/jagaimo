use super::{CommandToken, TokenizedCommand};
use crate::parse::alias_token;
use crate::parse::{AliasToken, Aliased, Aliases, CommandRule, CommandStack, Rules};
use syn::Ident;

impl CommandStack {
    pub fn tokenize_commands(&mut self) -> Vec<TokenizedCommand> {
        let cmds = self.take_rules().take_commands();
        let als = self.take_aliases().into_aliases();

        cmds.into_iter()
            .map(|c| (c, als.as_slice()).into())
            .collect()
    }
}
