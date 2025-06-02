use std::env::{current_dir as cwd, var};
use std::fs::read_to_string;

// this is crate wide public so that
// if I needed to get the manifest and reuse it multiple times,
// i would use this with ref returning fns
pub(crate) struct ResolveCrate {
    // running in workspace or not
    // is_workspace: bool,
    // name of the crate being compiled
    who: String,
    // manifest of the compile_who crate
    // manifest: String,
}

impl ResolveCrate {
    pub fn new() -> Self {
        let manifest = std::fs::read_to_string("Cargo.toml").unwrap();
        if manifest.contains("[workspace]") {
            let who = std::env::var("CARGO_CRATE_NAME").unwrap();
            Self { who }
        } else {
            Self {
                who: ".".to_owned(),
            }
        }
    }

    pub fn read_manifest(self) -> ReadManifest {
        self.into()
    }

    pub fn read_help(self) -> ReadHelp {
        self.into()
    }

    fn who(&self) -> &str {
        &self.who
    }
}

pub struct ReadHelp {
    help: String,
}

impl From<ResolveCrate> for ReadHelp {
    fn from(resolver: ResolveCrate) -> Self {
        Self {
            help: std::fs::read_to_string(resolver.who + "/help.toml").unwrap(),
        }
    }
}

impl ReadHelp {
    pub fn into_table(self) -> toml::Table {
        self.help.parse::<toml::Table>().unwrap()
    }
}

pub(crate) struct ReadManifest {
    manifest: String,
}

impl From<ResolveCrate> for ReadManifest {
    fn from(resolver: ResolveCrate) -> Self {
        Self {
            manifest: std::fs::read_to_string(resolver.who + "/Cargo.toml").unwrap(),
        }
    }
}

impl ReadManifest {
    pub fn crate_name_version(&self) -> [String; 2] {
        let mut iter = self
            .manifest
            .lines()
            .filter(|l| l.starts_with("name = ") || l.starts_with("version = "))
            .map(|l| l.to_owned());

        [iter.next().unwrap(), iter.next().unwrap()]
    }

    pub(crate) fn crate_name(&self) -> &str {
        self.manifest
            .lines()
            .find(|l| l.starts_with("name = "))
            .map(|l| &l[8..l.len() - 1])
            .unwrap()
    }

    pub(crate) fn crate_version(&self) -> &str {
        self.manifest
            .lines()
            .find(|l| l.starts_with("version = "))
            .map(|l| &l[10..l.len() - 1])
            .unwrap()
    }
}
