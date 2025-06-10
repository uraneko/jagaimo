use std::env;
use std::fs;

use crate::ReadManifest;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ResolveCrate {
    who: String,
}

impl ResolveCrate {
    pub fn new() -> Self {
        let who = if let Ok(who) = env::var("CARGO_MANIFEST_DIR") {
            who + "/"
        } else if let Ok(who) = env::var("CARGO_PKG_NAME") {
            let cwd = env::current_dir().unwrap();
            cwd.into_os_string()
                .into_string()
                .expect("if this was not a proper string, the earlier var calls would have failed")
                + "/"
                + &who
                + "/"
        } else {
            let m = fs::read_to_string("Cargo.toml").unwrap();
            if let (true, Ok(p)) = (m.contains("[workspace]"), env::var("CARGO_CRATE_NAME")) {
                p + "/"
            } else {
                "./".into()
            }
        };

        Self { who }
    }

    pub fn read_manifest(&self) -> ReadManifest {
        self.into()
    }

    pub fn into_manifest(self) -> ReadManifest {
        self.into()
    }

    pub fn as_str(&self) -> &str {
        &self.who
    }

    pub fn to_string(&self) -> String {
        self.who.clone()
    }

    pub fn into_string(self) -> String {
        self.who
    }

    pub fn into_string_suffixed<T: AsRef<str>>(self, suffix: T) -> String {
        self.who + suffix.as_ref()
    }

    pub fn to_string_suffixed(&self, s: impl AsRef<str>) -> String {
        self.who.clone() + s.as_ref()
    }

    pub fn prefix_str(&self, path: impl AsRef<str>) -> String {
        self.as_str().chars().chain(path.as_ref().chars()).collect()
    }
}

impl AsRef<str> for ResolveCrate {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for ResolveCrate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "crate foudn at <{}>", self.who)
    }
}
