use resolve_calling_crate::ResolveCrate;
use std::collections::HashMap;

use crate::styled::Styled;

enum Scope {}

pub trait Help {
    // formats the entire help message of self
    // and returns it
    fn help(scope: Scope) -> String {
        let stls = Self::styles();
        let hdr = stls.get("hdr").unwrap();
        let desc = Self::description();
        let links = Self::links(&stls).unwrap_or("".into());
        let usage = Self::usage(&stls);
        let spaces = Self::spaces(&stls)
            .map(|s| hdr.fmt("Spaces:") + &s)
            .unwrap_or("".into());
        let ops = Self::ops(&stls)
            .map(|s| hdr.fmt("Operations:") + &s)
            .unwrap_or("".into());
        let flags = Self::flags(&stls)
            .map(|s| hdr.fmt("Flags:") + &s)
            .unwrap_or("".into());

        format!(
            "{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}",
            desc, links, usage, flags, ops, spaces
        )
    }

    fn styles() -> HashMap<&'static str, Styled> {
        HashMap::from([
            ("base", Styled::default().color("y").effects(["b", "ul"])),
            ("flag", Styled::default().color("r").effect("b")),
            ("op", Styled::default().color("g").effect("b")),
            ("space", Styled::default().color("blue").effect("bold")),
            ("clear", Styled::default().effect("clear")),
            ("hdr", Styled::default().color("mgnt").effects(["b", "itl"])),
        ])
    }

    // returns the usage string of this scope
    fn usage(stls: &HashMap<&str, Styled>) -> String;

    // returns the url links provided in self's help message
    fn links(stls: &HashMap<&str, Styled>) -> Option<String> {
        None
    }

    fn flags(stls: &HashMap<&str, Styled>) -> Option<String> {
        None
    }

    fn ops(stls: &HashMap<&str, Styled>) -> Option<String> {
        None
    }

    fn spaces(stls: &HashMap<&str, Styled>) -> Option<String> {
        None
    }

    // returns the description of this scope, if any
    // if none then defaults to returning self.named_scope()
    fn description() -> String;

    // returns the cli tool version
    // returns option because version only makes sense on the root namespace
    fn version() -> String {
        let [mut name, version] = ResolveCrate::new().read_manifest().crate_name_version();

        name.push_str(" ");
        name.push_str(&version);

        name
    }
}

fn gen_delim(k: &mut String) {
    let len = k.len();
    let delim = 20 - len;
    k.push_str(&(0..delim).into_iter().map(|_| ' ').collect::<String>());
}

fn format_table(t: HashMap<String, String>, stl: &mut Styled) -> String {
    t.into_iter()
        .map(|(k, v)| (stl.fmt(k), v))
        .map(|(mut k, v)| {
            gen_delim(&mut k);
            k + &v
        })
        .fold(String::new(), |acc, l| acc + &l + "\n")
}
