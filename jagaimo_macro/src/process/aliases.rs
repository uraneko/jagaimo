use super::{CommandToken, TokenizedCommand, TokenizedCommands};
use crate::parse::alias_token;
use crate::parse::{AliasToken, Aliased, Aliases, CommandRule, CommandStack, Rules};
use syn::Ident;

impl CommandStack {
    // generates aliases when auto alias is on
    pub fn generate_auto_aliases(&mut self) {
        if !self.attrs().auto_alias() {
            return;
        }

        let mut als = std::mem::take(self.aliases_mut());
        let cmds = self.rules_ref().commands();
        cmds.iter().for_each(|cr| {
            if let Some(s) = cr.space() {
                generate_aliased(s, 's', &mut als);
            }
            if let Some(o) = cr.op() {
                generate_aliased(o, 'o', &mut als);
            }
            if let Some(f) = cr.flags() {
                f.iter()
                    .for_each(|f| generate_aliased(f.ident(), 'f', &mut als))
            }
        });

        self.set_aliases(&mut als);
    }

    pub fn resolve_aliases(&mut self) {
        self.rules_mut().dedup_commands();

        let mut als = std::mem::take(self.aliases_mut());
        als = Aliases::from_values(
            als.into_iter()
                .filter(|a| self.rules_ref().matches_commands(a))
                .collect(),
        );
        self.set_aliases(&mut als);

        self.generate_auto_aliases();
    }
}

fn generate_aliased(i: &Ident, s: char, als: &mut Aliases) {
    let t = alias_token!(s, i);
    let a = t.alias();
    if a.is_none() {
        return;
    }

    let aliased = Aliased::from_values(t, a.unwrap());
    if !als.contains(&aliased) {
        als.push(aliased);
    }
}
